use run::input::InputCart as Cart;
use run::input::InputCartLines as CartLine;
use run::input::InputCartLinesMerchandise::ProductVariant;
use run::input::InputCartLinesMerchandiseOnProductVariant;
use run::output::*;
use shopify_function::prelude::*;
use shopify_function::Result;
use std::collections::HashMap;

const PARENT_PRODUCT_ID: &str = "gid://shopify/ProductVariant/42205904568380";
const SNOWBOARD_PRODUCT_ID: &str = "gid://shopify/ProductVariant/41272251809852";
const WAX_PRODUCT_ID: &str = "gid://shopify/ProductVariant/42205916954684";
const PRICE_ADJUSTMENT_PERCENTAGE: f64 = 50.0;

#[derive(Clone, Debug, PartialEq)]
struct ComponentParent {
    pub id: ID,
    pub component_reference: Vec<ID>,
    pub component_quantities: Vec<i64>,
    pub price_adjustment: Option<f64>,
}

#[shopify_function_target(query_path = "src/run.graphql", schema_path = "schema.graphql")]
fn run(input: input::ResponseData) -> Result<output::FunctionRunResult> {
    let cart_operations: Vec<CartOperation> = get_merge_cart_operations(&input.cart).collect();

    Ok(output::FunctionRunResult {
        operations: cart_operations,
    })
}

// merge operation logic

fn get_merge_cart_operations(cart: &Cart) -> impl Iterator<Item = CartOperation> + '_ {
    let merge_parent_definitions = get_merge_parent_definitions(cart);

    let mut cart_lines: Vec<CartLine> = Vec::new();
    cart.lines
        .iter()
        .for_each(|line| cart_lines.push(line.clone()));

    merge_parent_definitions
        .into_iter()
        .filter_map(move |definition| {
            let components_in_cart = get_components_in_cart(&mut cart_lines, &definition);
            (components_in_cart.len() == definition.component_reference.len()).then(|| {
                let cart_lines: Vec<CartLineInput> = components_in_cart
                    .into_iter()
                    .map(|component| CartLineInput {
                        cart_line_id: component.cart_line_id,
                        quantity: component.quantity,
                    })
                    .collect();

                let price = Some(PriceAdjustment {
                    percentage_decrease: Some(PriceAdjustmentValue {
                        value: Decimal(PRICE_ADJUSTMENT_PERCENTAGE),
                    }),
                });

                let merge_operation = MergeOperation {
                    parent_variant_id: definition.id,
                    title: None,
                    cart_lines,
                    image: None,
                    price,
                    attributes: None,
                };

                CartOperation::Merge(merge_operation)
            })
        })
}

fn get_components_in_cart(
    cart_lines: &mut Vec<CartLine>,
    definition: &ComponentParent,
) -> Vec<CartLineInput> {
    let line_results: Vec<CartLineInput> = definition
        .component_reference
        .iter()
        .zip(definition.component_quantities.iter())
        .filter_map(|(reference, &quantity)| {
            cart_lines.iter().find_map(move |line| {
                matches!(
                    &line.merchandise,
                    ProductVariant(merchandise) if reference == &merchandise.id && line.quantity >= quantity,
                ).then(|| CartLineInput { cart_line_id: line.id.clone(), quantity })
            })
        })
        .collect();

    update_cart_lines_from_function_result(cart_lines, line_results.clone());

    line_results
}

fn update_cart_lines_from_function_result(
    cart_lines: &mut Vec<CartLine>,
    line_results: Vec<CartLineInput>,
) {
    let mut cart_line_tracker: HashMap<String, i64> = cart_lines
        .iter()
        .map(|cart_line| (cart_line.id.clone(), cart_line.quantity))
        .collect();

    for line_result in line_results.iter() {
        if let Some(target_cart_line) = cart_lines
            .iter()
            .find(|cart_line| cart_line.id == line_result.cart_line_id)
        {
            let new_quantity = cart_line_tracker[&target_cart_line.id] - line_result.quantity;
            cart_line_tracker.insert(target_cart_line.id.clone(), new_quantity);
        }
    }

    cart_lines.retain(|line| cart_line_tracker[&line.id] > 0);

    for cart_line in cart_lines.iter_mut() {
        if cart_line_tracker[&cart_line.id] > 0 {
            cart_line.quantity = cart_line_tracker[&cart_line.id];
        }
    }
}

fn get_merge_parent_definitions(cart: &Cart) -> Vec<ComponentParent> {
    let mut merge_parent_definitions: Vec<ComponentParent> = Vec::new();

    for line in cart.lines.iter() {
        if let ProductVariant(merchandise) = &line.merchandise {
            merge_parent_definitions.extend(get_component_parents(merchandise));
        }
    }

    merge_parent_definitions
}

fn get_component_parents(
    variant: &InputCartLinesMerchandiseOnProductVariant,
) -> impl Iterator<Item = ComponentParent> {
    // Check if the variant is the Snowboard or Wax product
    if variant.id == SNOWBOARD_PRODUCT_ID || variant.id == WAX_PRODUCT_ID {
        Some(ComponentParent {
            id: PARENT_PRODUCT_ID.to_string(),
            component_reference: vec![SNOWBOARD_PRODUCT_ID.to_string(), WAX_PRODUCT_ID.to_string()],
            component_quantities: vec![1, 1],
            price_adjustment: Some(PRICE_ADJUSTMENT_PERCENTAGE),
        })
        .into_iter()
    } else {
        None.into_iter()
    }
}