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

    let discounts: Vec<run::output::Discount> = get_discounts(&input.cart.delivery_groups);

    Ok(run::output::FunctionRunResult {
        discounts
    })
}

fn get_discounts(groups: &Vec<run::input::InputCartDeliveryGroups>) -> Vec<run::output::Discount> {
    let mut result: Vec<run::output::Discount> = Vec::new();
    

    for group in groups.iter() {
     
        //This is the code to reduce all options even if not selected
       // if group.id == "gid://shopify/CartDeliveryGroup/84745027644" {
        for option in &group.delivery_options {
            if (option.title.as_ref().map(String::as_str) == Some("Express") && option.cost.amount != Decimal::from_str("15.0").unwrap())
            || (option.title.as_ref().map(String::as_str) == Some("Standard") && option.cost.amount != Decimal::from_str("10.0").unwrap()) {
                let target = run::output::Target::DeliveryOption(run::output::DeliveryOptionTarget {
                    handle: option.handle.clone(),
                });
           




        
        //This is the code for reducing the first options even if not selected

        /*
        if let Some(first_option) = group.delivery_options.first() {
            let target = run::output::Target::DeliveryOption(run::output::DeliveryOptionTarget {
                handle: first_option.handle.clone(),
            });
            */

            //This is the code for reducing a selected option
/*
        if let Some(selected_option) = &group.selected_delivery_option {
            let target = run::output::Target::DeliveryOption(run::output::DeliveryOptionTarget {
                handle: selected_option.handle.clone(),
            });
            */

            let discount = run::output::Discount {
                message: Some(option.title.clone().unwrap_or("No title".to_string())),
                targets: vec![target],
                value: run::output::Value::Percentage(run::output::Percentage {
                    value: Decimal::from_str("50.0").unwrap()
                }),
            };
              
            result.push(discount);
           }

        
        }
    }

    result
}