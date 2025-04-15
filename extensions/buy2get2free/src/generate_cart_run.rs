use shopify_function::prelude::*;
use shopify_function::Result;

use cart_lines_discounts_generate_run::output::{
    CartLinesDiscountsGenerateRunResult, CartLineTarget, CartOperation, 
    ProductDiscountCandidate, ProductDiscountCandidateTarget, ProductDiscountCandidateValue,
    ProductDiscountSelectionStrategy, ProductDiscountsAddOperation, Percentage,
};

use cart_lines_discounts_generate_run::input::ResponseData;

#[shopify_function_target(
    target = "cartLinesDiscountsGenerateRun",
    query_path = "src/generate_cart_run.graphql",
    schema_path = "schema.graphql"
)]
fn generate_cart_run(input: ResponseData) -> Result<CartLinesDiscountsGenerateRunResult> {
    // Calculate total quantity in cart
    let total_quantity: i64 = input.cart.lines.iter()
        .map(|line| line.quantity)
        .sum();

    // Only apply discount if there are at least 4 items
    if total_quantity < 4 {
        return Ok(CartLinesDiscountsGenerateRunResult { operations: vec![] });
    }

    // Calculate sets of 4 items (buy 2, get 2 free)
    let complete_sets = total_quantity / 4;
    let free_items = complete_sets * 2; // 2 free items per complete set

    // If no complete sets, return empty result
    if complete_sets == 0 {
        return Ok(CartLinesDiscountsGenerateRunResult { operations: vec![] });
    }

    // Create a vector of (line_id, price_per_item, quantity) tuples
    let mut line_prices: Vec<(String, f64, i64)> = input.cart.lines.iter()
        .map(|line| {
            let price = line.cost.amount_per_quantity.amount.0;
            (line.id.clone(), price, line.quantity)
        })
        .collect();

    // Sort by price (lowest first)
    line_prices.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    let mut remaining_free_items = free_items;
    let mut discount_targets = vec![];

    // Allocate free items to the cheapest products
    for (line_id, _, quantity) in line_prices {
        if remaining_free_items <= 0 {
            break;
        }

        let items_to_discount = std::cmp::min(remaining_free_items, quantity);
        remaining_free_items -= items_to_discount;

        discount_targets.push(ProductDiscountCandidateTarget::CartLine(CartLineTarget {
            id: line_id,
            quantity: Some(items_to_discount),
        }));
    }

    // Create the discount operation
    let operations = vec![CartOperation::ProductDiscountsAdd(ProductDiscountsAddOperation {
        selection_strategy: ProductDiscountSelectionStrategy::FIRST,
        candidates: vec![ProductDiscountCandidate {
            targets: discount_targets,
            message: Some(format!("Buy 2, Get 2 Free - {} items free", free_items)),
            value: ProductDiscountCandidateValue::Percentage(Percentage {
                value: Decimal(100.0),
            }),
            associated_discount_code: None,
        }],
    })];

    Ok(CartLinesDiscountsGenerateRunResult { operations })
}

#[cfg(test)]
mod tests {
    use super::*;
    use shopify_function::{run_function_with_input, Result};

    #[test]
    fn test_no_discount_for_single_item() -> Result<()> {
        let result = run_function_with_input(
            generate_cart_run,
            r#"
                {
                    "cart": {
                        "lines": [
                            {
                                "id": "gid://shopify/CartLine/1",
                                "quantity": 1,
                                "cost": {
                                    "amountPerQuantity": {
                                        "amount": "10.00"
                                    }
                                }
                            }
                        ]
                    }
                }
            "#,
        )?;

        assert_eq!(result.operations.len(), 0);
        Ok(())
    }

    #[test]
    fn test_no_discount_for_two_items() -> Result<()> {
        let result = run_function_with_input(
            generate_cart_run,
            r#"
                {
                    "cart": {
                        "lines": [
                            {
                                "id": "gid://shopify/CartLine/1",
                                "quantity": 2,
                                "cost": {
                                    "amountPerQuantity": {
                                        "amount": "10.00"
                                    }
                                }
                            }
                        ]
                    }
                }
            "#,
        )?;

        assert_eq!(result.operations.len(), 0);
        Ok(())
    }

    #[test]
    fn test_buy_two_get_two_free() -> Result<()> {
        let result = run_function_with_input(
            generate_cart_run,
            r#"
                {
                    "cart": {
                        "lines": [
                            {
                                "id": "gid://shopify/CartLine/1",
                                "quantity": 2,
                                "cost": {
                                    "amountPerQuantity": {
                                        "amount": "20.00"
                                    }
                                }
                            },
                            {
                                "id": "gid://shopify/CartLine/2",
                                "quantity": 2,
                                "cost": {
                                    "amountPerQuantity": {
                                        "amount": "10.00"
                                    }
                                }
                            }
                        ]
                    }
                }
            "#,
        )?;

        assert_eq!(result.operations.len(), 1);
        if let CartOperation::ProductDiscountsAdd(op) = &result.operations[0] {
            assert_eq!(op.candidates.len(), 1);
            assert_eq!(op.candidates[0].targets.len(), 1);
            if let ProductDiscountCandidateTarget::CartLine(target) = &op.candidates[0].targets[0] {
                assert_eq!(target.id, "gid://shopify/CartLine/2");
                assert_eq!(target.quantity, Some(2));
            }
        }
        Ok(())
    }

    #[test]
    fn test_buy_four_get_four_free() -> Result<()> {
        let result = run_function_with_input(
            generate_cart_run,
            r#"
                {
                    "cart": {
                        "lines": [
                            {
                                "id": "gid://shopify/CartLine/1",
                                "quantity": 4,
                                "cost": {
                                    "amountPerQuantity": {
                                        "amount": "20.00"
                                    }
                                }
                            },
                            {
                                "id": "gid://shopify/CartLine/2",
                                "quantity": 4,
                                "cost": {
                                    "amountPerQuantity": {
                                        "amount": "10.00"
                                    }
                                }
                            }
                        ]
                    }
                }
            "#,
        )?;

        assert_eq!(result.operations.len(), 1);
        if let CartOperation::ProductDiscountsAdd(op) = &result.operations[0] {
            assert_eq!(op.candidates.len(), 1);
            assert_eq!(op.candidates[0].targets.len(), 1);
            if let ProductDiscountCandidateTarget::CartLine(target) = &op.candidates[0].targets[0] {
                assert_eq!(target.id, "gid://shopify/CartLine/2");
                assert_eq!(target.quantity, Some(4));
            }
        }
        Ok(())
    }
}
