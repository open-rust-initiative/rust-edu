use super::{
    super::datatype::symbol::Symbol,
    ast::*,
};

#[derive(Debug, Clone)]
pub enum Statement {
    Select {
        distinct: bool,
        projections: Column,
        table: Vec<(Expression, Option<Expression>)>,
        filter: Option<Condition>,
        group_by: Column,
        having: Option<Condition>,
        order_by: Option<Vec<(String, Sort)>>
    },
}

#[derive(Debug, Clone)]
pub enum Column {
    AllColumns,
    Columns(Vec<(Expression, Option<Expression>)>),
}

#[derive(Debug, Clone)]
pub enum Sort {
    ASC,
    DESC
}

#[derive(Debug, Clone)]
pub enum Condition {
    And {
        left: Box<Condition>,
        right: Box<Condition>,
    },
    Or {
        left: Box<Condition>,
        right: Box<Condition>,
    },
    Not(Box<Condition>),
    Comparison {
        left: Expression,
        operator: Symbol,
        right: Expression,
    }
}

#[derive(Debug, Clone)]
pub struct Expression {
    pub ast: ASTNode,
}

impl Expression {
    pub fn new(left: ASTNode, symbol: Symbol, right: ASTNode) -> Self {
        Self { 
            ast: ASTNode::new(
                NodeType::Symbol(symbol),
                Some(Box::new(left)),
                Some(Box::new(right))
            )
        }
    }

    pub fn new_with_ast(ast: ASTNode) -> Self {
        Self { ast }
    }

    pub fn new_with_symbol(s: Symbol) -> Self {
        Self { ast: ASTNode::default(NodeType::Symbol(s)) }
    }

    pub fn new_left(node: NodeType) -> Self {
        Self { ast: ASTNode::new_node(node) }
    }

    pub fn new_unary_op(s: Symbol, expr: ASTNode) -> Self {
        Self { 
            ast: ASTNode::new(
                NodeType::Symbol(s),
                Some(Box::new(expr)),
                None
            ) 
        }
    }
}