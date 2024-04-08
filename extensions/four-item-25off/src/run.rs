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

    //No nessasary however this is an extra condition that a customer has a tag that must be true before the discount applies.
    let vip = match input.cart.buyer_identity {
        Some(identity) => match identity.customer {
            Some(customer) => match customer.has_any_tag {
                true => true,
                false => false,
            },
            None => {
                eprintln!("No tag for VIP");
                return Ok(no_discount);
            }
        },
        None => {
            eprintln!("No cart buyer identity found");
            return Ok(no_discount);
        }
    };

    if vip == false {
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
        //Add apply logic here, in this case we are applying to all line items. But you can tailor this to specific product of collections depending on the conditions you want the discount code to work for.
        if line.quantity == 4 {
            match &line.merchandise {
                run::input::InputCartLinesMerchandise::ProductVariant(variant) => {
                    let target = run::output::Target::ProductVariant(run::output::ProductVariantTarget {
                        id: variant.id.clone(),
                        quantity: None,
                    });

                    // Calculate the discount amount 
                    //This example is discounting a fixed amount 15% 

                   // let ppp = line.cost.amount_per_quantity.amount; // get current price
                   // let ppp_f64 = ppp.as_f64(); // convert to floating point
                   // let discount_amount = (ppp_f64 * 0.15).ceil(); //round up because you want the highest $ amount discount off.
                   
                    let discount = run::output::Discount {
                        message: None,
                        targets: vec![target],
                        value: run::output::Value::Percentage(run::output::Percentage {
                            value: Decimal::from_str("25.0").unwrap()
                    
                        }),
                    };
                  
                    result.push(discount);
                },
                _ => {}
            }
        }
    }

    return result;
}