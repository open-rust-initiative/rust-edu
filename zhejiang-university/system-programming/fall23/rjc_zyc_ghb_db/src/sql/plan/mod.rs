pub mod planner;

use self::planner::Planner;

use super::{engine::Transaction, schema::{catalog::Catalog, table::Table}, types::expression::Expression, execution::{ResultSet, Executor}, parser::ast::Statement};
use crate::error::Result;

use serde_derive::{Deserialize, Serialize};
use std::fmt::{self, Display};

/// SQL Plan
#[derive(Debug)]
pub struct Plan(pub Node);

impl Display for Plan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Plan {
    /// 从 AST 构造 Planner
    pub fn build<C: Catalog>(statement: Statement, catalog: &mut C) -> Result<Self> {
        Planner::new(catalog).build(statement)
    }

    /// 执行计划
    pub fn execute<T: Transaction + 'static>(self, txn: &mut T) -> Result<ResultSet> {
        <dyn Executor<T>>::build(self.0).execute(txn)
    }

    /// 优化，例如谓词下推等等，目前暂时不优化
    pub fn optimize<C: Catalog>(self, _catalog: &mut C) -> Result<Self> {
        // let mut root = self.0;
        Ok(Plan(self.0))
    }
}

/// A plan node
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Node {
    CreateTable {
        schema: Table,
    },
    Delete {
        table: String,
        source: Box<Node>,
    },
    DropTable {
        table: String,
    },
    Filter {
        source: Box<Node>,
        predicate: Expression,
    },
    Insert {
        table: String,
        columns: Vec<String>,
        expressions: Vec<Vec<Expression>>,
    },
    Projection {
        source: Box<Node>,
        expressions: Vec<(Expression, Option<String>)>,
    },
    Scan {
        table: String,
        alias: Option<String>,
        filter: Option<Expression>,
    },
    Update {
        table: String,
        source: Box<Node>,
        expressions: Vec<(usize, Option<String>, Expression)>,
    },
    Nothing,
}

impl Node {
    /// Recursively transforms nodes by applying functions before and after descending.
    pub fn transform<B, A>(mut self, before: &B, after: &A) -> Result<Self>
    where
        B: Fn(Self) -> Result<Self>,
        A: Fn(Self) -> Result<Self>,
    {
        self = before(self)?;
        self = match self {
            n @ Self::CreateTable { .. }
            | n @ Self::DropTable { .. }
            | n @ Self::Insert { .. }
            | n @ Self::Nothing
            | n @ Self::Scan { .. } => n,
            Self::Delete { table, source } => {
                Self::Delete { table, source: source.transform(before, after)?.into() }
            }
            Self::Filter { source, predicate } => {
                Self::Filter { source: source.transform(before, after)?.into(), predicate }
            }
            Self::Projection { source, expressions } => {
                Self::Projection { source: source.transform(before, after)?.into(), expressions }
            }
            Self::Update { table, source, expressions } => {
                Self::Update { table, source: source.transform(before, after)?.into(), expressions }
            }
        };
        after(self)
    }

    /// Transforms all expressions in a node by calling .transform() on them with the given closure.
    pub fn transform_expressions<B, A>(self, before: &B, after: &A) -> Result<Self>
    where
        B: Fn(Expression) -> Result<Expression>,
        A: Fn(Expression) -> Result<Expression>,
    {
        Ok(match self {
            n @ Self::CreateTable { .. }
            | n @ Self::Delete { .. }
            | n @ Self::DropTable { .. }
            | n @ Self::Nothing
            | n @ Self::Scan { filter: None, .. } => n,

            Self::Filter { source, predicate } => {
                Self::Filter { source, predicate: predicate.transform(before, after)? }
            }
            Self::Insert { table, columns, expressions } => Self::Insert {
                table,
                columns,
                expressions: expressions
                    .into_iter()
                    .map(|exprs| exprs.into_iter().map(|e| e.transform(before, after)).collect())
                    .collect::<Result<_>>()?,
            },
            Self::Projection { source, expressions } => Self::Projection {
                source,
                expressions: expressions
                    .into_iter()
                    .map(|(e, l)| Ok((e.transform(before, after)?, l)))
                    .collect::<Result<_>>()?,
            },
            Self::Scan { table, alias, filter: Some(filter) } => {
                Self::Scan { table, alias, filter: Some(filter.transform(before, after)?) }
            }
            Self::Update { table, source, expressions } => Self::Update {
                table,
                source,
                expressions: expressions
                    .into_iter()
                    .map(|(i, l, e)| e.transform(before, after).map(|e| (i, l, e)))
                    .collect::<Result<_>>()?,
            },
        })
    }

    // Displays the node, where prefix gives the node prefix.
    pub fn format(&self, mut indent: String, root: bool, last: bool) -> String {
        let mut s = indent.clone();
        if !last {
            s += "├─ ";
            indent += "│  "
        } else if !root {
            s += "└─ ";
            indent += "   ";
        }
        match self {
            Self::CreateTable { schema } => {
                s += &format!("CreateTable: {}\n", schema.name);
            }
            Self::Delete { source, table } => {
                s += &format!("Delete: {}\n", table);
                s += &source.format(indent, false, true);
            }
            Self::DropTable { table } => {
                s += &format!("DropTable: {}\n", table);
            }
            Self::Filter { source, predicate } => {
                s += &format!("Filter: {}\n", predicate);
                s += &source.format(indent, false, true);
            }
            Self::Insert { table, columns: _, expressions } => {
                s += &format!("Insert: {} ({} rows)\n", table, expressions.len());
            }
            Self::Projection { source, expressions } => {
                s += &format!(
                    "Projection: {}\n",
                    expressions
                        .iter()
                        .map(|(expr, _)| expr.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                s += &source.format(indent, false, true);
            }
            Self::Scan { table, alias, filter } => {
                s += &format!("Scan: {}", table);
                if let Some(alias) = alias {
                    s += &format!(" as {}", alias);
                }
                if let Some(expr) = filter {
                    s += &format!(" ({})", expr);
                }
                s += "\n";
            }
            Self::Update { source, table, expressions } => {
                s += &format!(
                    "Update: {} ({})\n",
                    table,
                    expressions
                        .iter()
                        .map(|(i, l, e)| format!(
                            "{}={}",
                            l.clone().unwrap_or_else(|| format!("#{}", i)),
                            e
                        ))
                        .collect::<Vec<_>>()
                        .join(",")
                );
                s += &source.format(indent, false, true);
            },
            Self::Nothing {} => {
                s += "Nothing\n";
            }
        };
        if root {
            s = s.trim_end().to_string()
        }
        s
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format("".into(), true, true))
    }
}