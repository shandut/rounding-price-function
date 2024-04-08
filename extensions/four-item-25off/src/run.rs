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
        discounts: vec![],
        discount_application_strategy: run::output::DiscountApplicationStrategy::ALL,
    };

    let vip = if let Some(identity) = input.cart.buyer_identity {
        if let Some(customer) = identity.customer {
            customer.has_any_tag
        } else {
            eprintln!("No tag for VIP");
            return Ok(no_discount);
        }
    } else {
        eprintln!("No cart buyer identity found");
        return Ok(no_discount);
    };

    if !vip {
        eprintln!("User is not VIP");
        return Ok(no_discount);
    }

    let discounts: Vec<run::output::Discount> = get_discounts(&input.cart.lines);

    Ok(run::output::FunctionRunResult {
        discounts,
        discount_application_strategy: run::output::DiscountApplicationStrategy::ALL,
    })
}

fn get_discounts(lines: &Vec<run::input::InputCartLines>) -> Vec<run::output::Discount> {
    let mut result: Vec<run::output::Discount> = Vec::new();

    for line in lines.iter() {
        if line.quantity == 4 {
            if let run::input::InputCartLinesMerchandise::ProductVariant(variant) = &line.merchandise {
                let target = run::output::Target::ProductVariant(run::output::ProductVariantTarget {
                    id: variant.id.clone(),
                    quantity: None,
                });

                let discount = run::output::Discount {
                    message: None,
                    targets: vec![target],
                    value: run::output::Value::Percentage(run::output::Percentage {
                        value: Decimal::from_str("25.0").unwrap()
                    }),
                };
              
                result.push(discount);
            }
        }
    }

    result  //returns result
}