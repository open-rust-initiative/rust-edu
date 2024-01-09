use crate::system::errors::Errors;
use sqlparser::ast::{Expr, Join, JoinOperator, TableFactor, TableWithJoins};

#[derive(Debug, Clone)]
pub struct JoinInfo {
    pub left_table: String,
    pub left_column: String,
    pub right_table: String,
    pub right_column: String,
}

#[derive(Debug, Clone)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    FullOuter,
}

#[derive(Debug, Clone)]
pub enum FromType {
    Join {
        join_type: JoinType,
        join_info: JoinInfo,
    },
    String {
        tb: String,
    },
}

impl FromType {
    pub fn new(joins: Vec<TableWithJoins>) -> Result<Vec<FromType>, Errors> {
        let mut join_data_vec: Vec<FromType> = Vec::new();
        for join in joins {
            let relation = join.clone().relation;
            let joins = join.clone().joins;
            let main_table_name = match &relation {
                TableFactor::Table { name, .. } => name.to_string(),
                _ => "".to_string(),
            };
            if joins.len() == 0 {
                if let TableFactor::Table { name, .. } = relation {
                    join_data_vec.push(FromType::String {
                        tb: name.to_string(),
                    })
                }
            }
            for j in joins {
                if let Join {
                    relation: TableFactor::Table { name, .. },
                    join_operator,
                } = j
                {
                    let (join_type, join_constraint) = match join_operator {
                        JoinOperator::Inner(constraint) => (JoinType::Inner, constraint),
                        JoinOperator::LeftOuter(constraint) => (JoinType::Left, constraint),
                        JoinOperator::RightOuter(constraint) => (JoinType::Right, constraint),
                        JoinOperator::FullOuter(constraint) => (JoinType::FullOuter, constraint),
                        _ => return Err(Errors::UnimplementedOperation),
                    };

                    if let sqlparser::ast::JoinConstraint::On(expr) = join_constraint {
                        if let Expr::BinaryOp { left, op: _, right } = expr {
                            if let (
                                Expr::CompoundIdentifier(left_cols),
                                Expr::CompoundIdentifier(right_cols),
                            ) = (*left, *right)
                            {
                                if left_cols.len() == 2 && right_cols.len() == 2 {
                                    let join_info = JoinInfo {
                                        left_table: main_table_name.clone(),
                                        left_column: left_cols[1].to_string(),
                                        right_table: name.to_string(),
                                        right_column: right_cols[1].to_string(),
                                    };

                                    let join_data = FromType::Join {
                                        join_type,
                                        join_info,
                                    };
                                    join_data_vec.push(join_data);
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(join_data_vec)
    }
}
