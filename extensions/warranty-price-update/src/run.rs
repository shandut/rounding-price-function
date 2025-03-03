use shopify_function::prelude::*;
use shopify_function::Result;
use run::input::InputCart as Cart;
use run::output::{
    CartOperation, FunctionRunResult, UpdateOperation, UpdateOperationPriceAdjustment,
    UpdateOperationPriceAdjustmentValue, UpdateOperationFixedPricePerUnitAdjustment,
};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;

#[shopify_function_target(query_path = "src/run.graphql", schema_path = "schema.graphql")]
fn run(input: run::input::ResponseData) -> Result<FunctionRunResult> {
    let cart_operations: Vec<CartOperation> = get_update_cart_operations(&input.cart);
    Ok(FunctionRunResult { operations: cart_operations })
}

fn get_update_cart_operations(cart: &Cart) -> Vec<CartOperation> {
    let mut result: Vec<CartOperation> = Vec::new();

    let mut total_price_of_ice_products = Decimal::ZERO;
    let mut warranty_line_id: Option<String> = None;

    let target_variant_id = "gid://shopify/ProductVariant/42179605397564";
  let wagyu_id= "gid://shopify/ProductVariant/42606826750012";
    for line in &cart.lines {
        if let Some(attributes) = &line.attribute {
            match &attributes.value {
                Some(value) => eprintln!("Attribute Value: {}", value),
                None => eprintln!("Attribute Value: Not found"),
            }
        }

        if let run::input::InputCartLinesMerchandise::ProductVariant(variant) = &line.merchandise {
            if let Some(title) = variant.title.as_deref() {
                eprintln!("Product title: {}", title);
            }

            // Accumulate total price for "Ice" product
            if variant.title.as_deref() == Some("Ice") {
                let quantity_decimal = Decimal::from_i64(line.quantity).unwrap_or(Decimal::ZERO);
                eprintln!("Quantity Decimal: {}", quantity_decimal);
                let line_total_price = line.cost.amount_per_quantity.amount * quantity_decimal;
                total_price_of_ice_products += line_total_price;
            }

            // Identify the warranty product line using the target variant ID
            if variant.id == target_variant_id {
                warranty_line_id = Some(line.id.clone());
            }

            if variant.id == wagyu_id {
              if let Some(attributes) = &line.attribute {
                  if let Some(attribute_value) = attributes.value.clone() {
                      // Assuming the attribute value can be parsed as a f64.
                      if let Ok(attribute_multiplier) = attribute_value.parse::<f64>() {
                          let adjustment_amount = line.cost.amount_per_quantity.amount * Decimal::from_f64(attribute_multiplier).unwrap_or(Decimal::ONE);
                          eprintln!(
                              "Wagyu product found on line {} with per-unit cost {} and adjustment {}",
                              line.id, line.cost.amount_per_quantity.amount, adjustment_amount
                          );

                          let price_adjustment_value = UpdateOperationPriceAdjustmentValue::FixedPricePerUnit(
                              UpdateOperationFixedPricePerUnitAdjustment {
                                  amount: adjustment_amount,
                              },
                          );

                          let price_adjustment = UpdateOperationPriceAdjustment {
                              adjustment: price_adjustment_value,
                          };

                          let update_operation = UpdateOperation {
                              cart_line_id: line.id.clone(),
                              price: Some(price_adjustment),
                              image: None,
                              title: None,
                          };

                          result.push(CartOperation::Update(update_operation));
                      } else {
                          eprintln!("Failed to parse attribute value as f64 for line {}", line.id);
                      }
                  }
              }
          }
      }
  }

    // Apply warranty adjustment of 5% if "Ice" products exist and a warranty line is found
    if total_price_of_ice_products > Decimal::ZERO {
        if let Some(warranty_id) = warranty_line_id {
            let five_percent = Decimal::from_f64_retain(0.05).unwrap_or(Decimal::ZERO);
            let new_warranty_price = total_price_of_ice_products * five_percent;
            eprintln!("Warranty price_adjustment: {}", new_warranty_price);

            let price_adjustment_value = UpdateOperationPriceAdjustmentValue::FixedPricePerUnit(
                UpdateOperationFixedPricePerUnitAdjustment {
                    amount: new_warranty_price,
                },
            );

            let price_adjustment = UpdateOperationPriceAdjustment {
                adjustment: price_adjustment_value,
            };

            let update_operation = UpdateOperation {
                cart_line_id: warranty_id,
                price: Some(price_adjustment),
                image: None,
                title: None,
            };

            result.push(CartOperation::Update(update_operation));
        }
    }

    result
}