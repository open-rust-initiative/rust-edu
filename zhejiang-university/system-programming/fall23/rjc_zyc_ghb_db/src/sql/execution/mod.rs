pub mod operator;

use derivative::Derivative;
use serde_derive::{Serialize, Deserialize};

use crate::error::{Error, Result};

use self::operator::{create_table::CreateTable, delete::Delete, drop_table::DropTable, scan::Scan, insert::Insert, projection::Projection, filter::Filter, update::Update, nothing::Nothing};

use super::{types::{Columns, Rows, Row, Value}, engine::Transaction, plan::Node};

/// A plan executor
pub trait Executor<T: Transaction> {
    /// Executes the executor, consuming it and returning a result set
    fn execute(self: Box<Self>, txn: &mut T) -> Result<ResultSet>;
}

impl<T: Transaction + 'static> dyn Executor<T> {
    /// Builds an executor for a plan node, consuming it
    pub fn build(node: Node) -> Box<dyn Executor<T>> {
        match node {
            Node::CreateTable { schema } => CreateTable::new(schema),
            Node::Delete { table, source } => Delete::new(table, Self::build(*source)),
            Node::DropTable { table } => DropTable::new(table),
            Node::Filter { source, predicate } => Filter::new(Self::build(*source), predicate),
            Node::Insert { table, columns, expressions } => {
                Insert::new(table, columns, expressions)
            }
            Node::Projection { source, expressions } => {
                Projection::new(Self::build(*source), expressions)
            }
            Node::Scan { table, filter, alias: _ } => Scan::new(table, filter),
            Node::Update { table, source, expressions } => Update::new(
                table,
                Self::build(*source),
                expressions.into_iter().map(|(i, _, e)| (i, e)).collect(),
            ),
            Node::Nothing => Nothing::new(),
        }
    }
}


/// executor 返回的 result
#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Debug, PartialEq)]
pub enum ResultSet {
    Begin {
        version: u64,
        read_only: bool,
    },
    Commit {
        version: u64,
    },
    Rollback {
        version: u64,
    },
    Create {
        count: u64,
    },
    Delete {
        count: u64,
    },
    Update {
        count: u64,
    },
    CreateTable {
        name: String,
    },
    DropTable {
        name: String,
    },
    Query {
        columns: Columns,
        #[derivative(Debug = "ignore")]
        #[derivative(PartialEq = "ignore")]
        #[serde(skip, default = "ResultSet::empty_rows")]
        rows: Rows,
    }
}

impl ResultSet {
    fn empty_rows() -> Rows {
        Box::new(std::iter::empty())
    }

    /// 从 query 的 result set iter 获得下一个 row
    pub fn into_row(self) -> Result<Row> {
        if let ResultSet::Query { mut rows, .. } = self {
            rows.next().transpose()?.ok_or_else(|| Error::Value("No rows returned".into()))
        } else {
            Err(Error::Value(format!("Not a query result: {:?}", self)))
        }
    }

    /// 将 row 的数据转化为 value
    pub fn into_value(self) -> Result<Value> {
        self.into_row()?.into_iter().next().ok_or_else(|| Error::Value("No value returned".into()))
    }
}
