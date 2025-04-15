use shopify_function::prelude::*;
use shopify_function::Result;

use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use std::str::FromStr;


#[derive(Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
struct Configuration {}

impl Configuration {
    fn from_str(value: &str) -> Self {
        serde_json::from_str(value).expect("Unable to parse configuration value from metafield")
    }
}

#[shopify_function_target(query_path = "src/run.graphql", schema_path = "schema.graphql")]
fn run(input: input::ResponseData) -> Result<output::FunctionRunResult> {
    let no_discount = output::FunctionRunResult {
        discounts: vec![],
        discount_application_strategy: output::DiscountApplicationStrategy::FIRST,
    };

    let vip = if let Some(identity) = input.cart.buyer_identity {
        if let Some(customer) = identity.customer {
            customer.has_any_tag
        } else {
            false
        }
    } else {
        false
    };

    if !vip {
        eprintln!("User is not VIP");
        return Ok(no_discount);
    }

    // Create a discount for VIP customers
    let discount = output::Discount {
        message: Some("Trade/Member Discount".to_string()),
        targets: vec![output::Target::OrderSubtotal(output::OrderSubtotalTarget {
            excluded_variant_ids: vec![],
        })],
        value: output::Value::Percentage(output::Percentage {
            value: Decimal::from_str("25.0").unwrap(),
        }),
        conditions: None,
    };

    Ok(output::FunctionRunResult {
        discounts: vec![discount],
        discount_application_strategy: output::DiscountApplicationStrategy::FIRST,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use shopify_function::{run_function_with_input, Result};

    #[test]
    fn test_result_contains_no_discounts() -> Result<()> {
        use run::output::*;

        let result = run_function_with_input(
            run,
            r#"
                {
                    "discountNode": {
                        "metafield": null
                    }
                }
            "#,
        )?;
        let expected = FunctionRunResult {
            discounts: vec![],
            discount_application_strategy: DiscountApplicationStrategy::FIRST,
        };
        assert_eq!(result, expected);
        Ok(())
    }
}
