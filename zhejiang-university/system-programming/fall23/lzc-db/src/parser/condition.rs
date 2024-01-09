use crate::parser::select::BinaryOpCus;
use crate::system::errors::Errors;
use sqlparser::ast::Expr::{BinaryOp, IsNull, Like};
use sqlparser::ast::{BinaryOperator, Expr};

#[derive(Debug)]
pub enum Condition {
    Comparison {
        left: String,
        op: BinaryOpCus,
        right: Option<String>,
    },
    Logical {
        left: Box<Condition>,
        op: BinaryOpCus,
        right: Box<Condition>,
    },
}

impl Condition {
    pub fn from_expr(expr: &Expr) -> Result<Condition, Errors> {
        match expr {
            BinaryOp { left, op, right } => {
                let expr_op = match op {
                    BinaryOperator::Gt => BinaryOpCus::Gt,
                    BinaryOperator::Lt => BinaryOpCus::Lt,
                    BinaryOperator::Eq => BinaryOpCus::Eq,
                    BinaryOperator::And => BinaryOpCus::And,
                    BinaryOperator::Or => BinaryOpCus::Or,
                    _ => return Err(Errors::UnimplementedOperation),
                };

                if expr_op == BinaryOpCus::And || expr_op == BinaryOpCus::Or {
                    let left_condition = Condition::from_expr(left)?;
                    let right_condition = Condition::from_expr(right)?;

                    Ok(Condition::Logical {
                        left: Box::new(left_condition),
                        op: expr_op,
                        right: Box::new(right_condition),
                    })
                } else {
                    Ok(Condition::Comparison {
                        left: left.to_string(),
                        op: expr_op,
                        right: Some(right.to_string()),
                    })
                }
            }
            IsNull(x) => Ok(Condition::Comparison {
                left: x.to_string(),
                op: BinaryOpCus::IsNull,
                right: None,
            }),
            Like { expr, pattern, .. } => Ok(Condition::Comparison {
                left: expr.to_string(),
                op: BinaryOpCus::Like,
                right: Some(pattern.to_string()),
            }),
            _ => Err(Errors::InvalidExpression),
        }
    }
}
