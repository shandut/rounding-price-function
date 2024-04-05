use run::input::InputCart as Cart;
use run::output::CartOperation;
use run::output::UpdateOperation;
use run::output::UpdateOperationPriceAdjustment;
use run::output::UpdateOperationPriceAdjustmentValue;
use run::output::UpdateOperationFixedPricePerUnitAdjustment;
use shopify_function::prelude::*;
use shopify_function::Result;

#[shopify_function_target(query_path = "src/run.graphql", schema_path = "schema.graphql")]
fn run(input: input::ResponseData) -> Result<output::FunctionRunResult> {
    let cart_operations: Vec<CartOperation> = get_update_cart_operations(&input.cart);

    Ok(output::FunctionRunResult {
        operations: cart_operations,
    })
}

fn get_update_cart_operations(cart: &Cart) -> Vec<CartOperation> {
    let mut result: Vec<CartOperation> = Vec::new();

    for line in cart.lines.iter() {
        // Check if the price has a decimal part
        if line.cost.amount_per_quantity.amount.fract() != 0.0 {
            let rounded_price = line.cost.amount_per_quantity.amount.floor();

            let price_adjustment_value = UpdateOperationPriceAdjustmentValue::FixedPricePerUnit(
                UpdateOperationFixedPricePerUnitAdjustment {
                    amount: Decimal(rounded_price),
                }
            );

            let price_adjustment = UpdateOperationPriceAdjustment {
                adjustment: price_adjustment_value,
            };

            let update_operation: UpdateOperation = UpdateOperation {
                cart_line_id: line.id.clone(),
                price: Some(price_adjustment),
                image: None,
                title: None,
            };

            result.push(CartOperation::Update(update_operation));
        }
    }

    return result;
}