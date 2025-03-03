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
const PRICE_ADJUSTMENT_PERCENTAGE: f64 = -50.0;

#[shopify_function_target(query_path = "src/run.graphql", schema_path = "schema.graphql")]
fn run(input: input::ResponseData) -> Result<output::FunctionRunResult> {
    let cart_operations: Vec<CartOperation> = get_merge_cart_operations(&input.cart).collect();

    Ok(output::FunctionRunResult {
        operations: cart_operations,
    })
}

// merge operation logic

fn get_merge_cart_operations(cart: &Cart) -> impl Iterator<Item = CartOperation> + '_ {
    let mut cart_lines: Vec<CartLine> = cart.lines.clone();

    let snowboard_line = cart_lines.iter().find(|line| {
        matches!(
            &line.merchandise,
            ProductVariant(merchandise) if merchandise.id == SNOWBOARD_PRODUCT_ID && line.quantity >= 1
        )
    });

    let wax_line = cart_lines.iter().find(|line| {
        matches!(
            &line.merchandise,
            ProductVariant(merchandise) if merchandise.id == WAX_PRODUCT_ID && line.quantity >= 1
        )
    });

    if let (Some(snowboard_line), Some(wax_line)) = (snowboard_line, wax_line) {
        let cart_lines = vec![
            CartLineInput {
                cart_line_id: snowboard_line.id.clone(),
                quantity: 1,
            },
            CartLineInput {
                cart_line_id: wax_line.id.clone(),
                quantity: 1,
            },
        ];

        let price = Some(PriceAdjustment {
            percentage_decrease: Some(PriceAdjustmentValue {
                value: Decimal(PRICE_ADJUSTMENT_PERCENTAGE),
            }),
        });

        let merge_operation = MergeOperation {
            parent_variant_id: PARENT_PRODUCT_ID.to_string(),
            title: None,
            cart_lines,
            image: None,
            price,
            attributes: None,
        };

        vec![CartOperation::Merge(merge_operation)].into_iter()
    } else {
        Vec::new().into_iter()
    }
}