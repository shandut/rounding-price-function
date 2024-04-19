use shopify_function::prelude::*;
use shopify_function::Result;
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use std::str::FromStr;

#[derive(Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
struct Configuration {}

#[shopify_function_target(query_path = "src/run.graphql", schema_path = "schema.graphql")]
fn run(input: run::input::ResponseData) -> Result<run::output::FunctionRunResult> {
    let no_discount = run::output::FunctionRunResult {
        discounts: vec![]
    };

    let discounts: Vec<run::output::Discount> = get_discounts(&input.cart.cost,&input.cart.delivery_groups);

    Ok(run::output::FunctionRunResult {
        discounts
    })
}

fn get_discounts(cart_costing: &run::input::InputCartCost, groups: &Vec<run::input::InputCartDeliveryGroups>) -> Vec<run::output::Discount> {
    let mut result: Vec<run::output::Discount> = Vec::new();
  
    let subtotal = cart_costing.subtotal_amount.amount;
    let tax = cart_costing.total_tax_amount.as_ref().map_or(Decimal::from_str("0.0").unwrap(), |tax_amount| tax_amount.amount);
 
    let threshold = subtotal + tax;
   


        for group in groups.iter() {
        
            //This is the code to reduce all options even if not selected
        // if group.id == "gid://shopify/CartDeliveryGroup/84745027644" {
            for option in &group.delivery_options {
            
                    let target = run::output::Target::DeliveryOption(run::output::DeliveryOptionTarget {
                        handle: option.handle.clone(),
                    });

                let discount = run::output::Discount {
                    message: Some("You Qualify for Free Shipping!".to_string()),
                    targets: vec![target],
                    value: run::output::Value::Percentage(run::output::Percentage {
                        value: Decimal::from_str("100.0").unwrap()
                    }),
                };
                
                result.push(discount);
            }

            
        }
    



    result
}