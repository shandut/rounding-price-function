use shopify_function::prelude::*;
use shopify_function::Result;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, PartialEq)]
struct Config {}

// Function to check if a string contains Chinese characters
fn contains_chinese_characters(text: &str) -> bool {
    text.chars().any(|c| {
        let value = c as u32;
        // Check for Chinese character ranges
        (value >= 0x4E00 && value <= 0x9FFF) || // CJK Unified Ideographs
        (value >= 0x3400 && value <= 0x4DBF) || // CJK Unified Ideographs Extension A
        (value >= 0x20000 && value <= 0x2A6DF)  // CJK Unified Ideographs Extension B
    })
}

#[shopify_function_target(query_path = "src/run.graphql", schema_path = "schema.graphql")]
fn run(input: input::ResponseData) -> Result<output::FunctionRunResult> {
    let mut errors = Vec::new();

    // Only validate at checkout completion time not at cart
    if input.buyer_journey.step == Some(input::BuyerJourneyStep::CHECKOUT_COMPLETION) {
        // Get the cart total amount
        let cart_total = input.cart.cost.total_amount.amount.as_f64();
        
        // Log the raw metafield data for debugging
        eprintln!("Raw validation metafield: {:?}", input.validation.metafield);
        
        // Get the minimum threshold from validation metafield, default to 1000 if not set
        let threshold = input
            .validation
            .metafield
            .and_then(|m| {
                eprintln!("Found metafield with value: {:?}", m.value);
                m.value.parse::<f64>().ok()
            })
            .unwrap_or_else(|| {
                eprintln!("No valid metafield found, using default threshold of 1000.0");
                1000.0
            });
        
        // Check if cart total is less than threshold
        if cart_total < threshold {
            errors.push(output::FunctionError {
                localized_message: format!("Minimum order total of ${:.2} is required to proceed with checkout", threshold),
                target: "$.cart".to_owned(),
            });
        }

        // Check for Chinese characters in delivery addresses
        let mut has_chinese_characters = false;
        for group in input.cart.delivery_groups.iter() {
            if let Some(address) = &group.delivery_address {
                // Check all available address fields
                let fields_to_check = [
                    address.address1.as_deref(),
                    address.address2.as_deref(),
                    address.city.as_deref(),
                    address.company.as_deref(),
                    address.first_name.as_deref(),
                    address.last_name.as_deref(),
                    address.name.as_deref(),
                ];

                if fields_to_check.iter()
                    .filter_map(|&x| x)
                    .any(contains_chinese_characters) {
                    has_chinese_characters = true;
                    break;
                }
            }
        }

        if has_chinese_characters {
            errors.push(output::FunctionError {
                localized_message: "Sorry, Chinese characters are not allowed in the shipping address".to_owned(),
                target: "$.cart".to_owned(),
            });
        }
    }

    Ok(output::FunctionRunResult { errors })
}

#[cfg(test)]
mod tests {
    use super::*;
    use shopify_function::{run_function_with_input, Result};

    #[test]
    fn test_contains_chinese_characters() {
        assert!(contains_chinese_characters("你好"));
        assert!(contains_chinese_characters("Hello你好"));
        assert!(!contains_chinese_characters("Hello"));
        assert!(!contains_chinese_characters("123"));
    }

    #[test]
    fn test_result_contains_error_when_chinese_characters_present() -> Result<()> {
        let result = run_function_with_input(
            run,
            r#"
                {
                    "cart": {
                        "cost": {
                            "totalAmount": {
                                "amount": "1500.00"
                            }
                        },
                        "deliveryGroups": [{
                            "deliveryAddress": {
                                "firstName": "你好",
                                "lastName": "Smith",
                                "address1": "123 Street",
                                "city": "Toronto"
                            }
                        }]
                    },
                    "buyerJourney": {
                        "step": "CHECKOUT_COMPLETION"
                    },
                    "validation": {
                        "metafield": {
                            "value": "1000"
                        }
                    }
                }
            "#,
        )?;

        assert!(!result.errors.is_empty());
        assert!(result.errors.iter().any(|e| e.localized_message.contains("Chinese characters are not allowed")));
        Ok(())
    }

    #[test]
    fn test_result_contains_error_when_total_below_minimum_at_checkout() -> Result<()> {
        use run::output::*;

        let result = run_function_with_input(
            run,
            r#"
                {
                    "cart": {
                        "cost": {
                            "totalAmount": {
                                "amount": "500.00"
                            }
                        }
                    },
                    "buyerJourney": {
                        "step": "CHECKOUT_COMPLETION"
                    },
                    "validation": {
                        "metafield": {
                            "value": "1000"
                        }
                    }
                }
            "#,
        )?;
        let expected = FunctionRunResult {
            errors: vec![FunctionError {
                localized_message: "Minimum order total of $1000.00 is required to proceed with checkout".to_owned(),
                target: "$.cart".to_owned(),
            }],
        };

        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_result_contains_no_errors_when_total_above_minimum_at_checkout() -> Result<()> {
        use run::output::*;

        let result = run_function_with_input(
            run,
            r#"
                {
                    "cart": {
                        "cost": {
                            "totalAmount": {
                                "amount": "1500.00"
                            }
                        }
                    },
                    "buyerJourney": {
                        "step": "CHECKOUT_COMPLETION"
                    },
                    "validation": {
                        "metafield": {
                            "value": "1000"
                        }
                    }
                }
            "#,
        )?;
        let expected = FunctionRunResult { errors: vec![] };

        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_result_contains_no_errors_during_cart_operations() -> Result<()> {
        use run::output::*;

        let result = run_function_with_input(
            run,
            r#"
                {
                    "cart": {
                        "cost": {
                            "totalAmount": {
                                "amount": "500.00"
                            }
                        }
                    },
                    "buyerJourney": {
                        "step": "CART_INTERACTION"
                    },
                    "validation": {
                        "metafield": {
                            "value": "1000"
                        }
                    }
                }
            "#,
        )?;
        let expected = FunctionRunResult { errors: vec![] };

        assert_eq!(result, expected);
        Ok(())
    }
}
