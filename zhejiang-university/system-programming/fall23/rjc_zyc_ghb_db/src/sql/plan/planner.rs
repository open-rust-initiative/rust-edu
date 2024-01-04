use crate::error::{Error, Result};
use crate::sql::parser::ast;
use crate::sql::schema::catalog::Catalog;
use crate::sql::schema::table::{Table, Column};
use crate::sql::types::Value;
use crate::sql::types::expression::Expression;

use std::collections::{HashMap, HashSet};

use super::{Plan, Node};

/// A query plan builder.
pub struct Planner<'a, C: Catalog> {
    catalog: &'a mut C,
}

impl<'a, C: Catalog> Planner<'a, C> {
    /// Creates a new planner.
    pub fn new(catalog: &'a mut C) -> Self {
        Self { catalog }
    }

    /// Builds a plan for an AST statement.
    pub fn build(&mut self, statement: ast::Statement) -> Result<Plan> {
        Ok(Plan(self.build_statement(statement)?))
    }

    /// Builds a plan node for a statement.
    fn build_statement(&self, statement: ast::Statement) -> Result<Node> {
        Ok(match statement {
            // Transaction control and explain statements should have been handled by session.
            ast::Statement::Begin { .. } | ast::Statement::Commit | ast::Statement::Rollback => {
                return Err(Error::Internal(format!(
                    "Unexpected transaction statement {:?}",
                    statement
                )))
            }

            ast::Statement::Explain(_) => {
                return Err(Error::Internal("Unexpected explain statement".into()))
            }

            // DDL statements (schema changes).
            ast::Statement::CreateTable { name, columns } => Node::CreateTable {
                schema: Table::new(
                    name,
                    columns
                        .into_iter()
                        .map(|c| {
                            let nullable = c.nullable.unwrap_or(!c.primary_key);
                            let default = match c.default {
                                Some(expr) => Some(self.evaluate_constant(expr)?),
                                None if nullable => Some(Value::Null),
                                None => None,
                            };
                            Ok(Column {
                                name: c.name,
                                datatype: c.datatype,
                                primary_key: c.primary_key,
                                nullable,
                                default,
                                index: c.index && !c.primary_key,
                                unique: c.unique || c.primary_key,
                                references: c.references,
                            })
                        })
                        .collect::<Result<_>>()?,
                )?,
            },

            ast::Statement::DropTable(table) => Node::DropTable { table },

            // DML statements (mutations).
            ast::Statement::Delete { table, r#where } => {
                let scope = &mut Scope::from_table(self.catalog.must_read_table(&table)?)?;
                Node::Delete {
                    table: table.clone(),
                    source: Box::new(Node::Scan {
                        table,
                        alias: None,
                        filter: r#where.map(|e| self.build_expression(scope, e)).transpose()?,
                    }),
                }
            }

            ast::Statement::Insert { table, columns, values } => Node::Insert {
                table,
                columns: columns.unwrap_or_default(),
                expressions: values
                    .into_iter()
                    .map(|exprs| {
                        exprs
                            .into_iter()
                            .map(|expr| self.build_expression(&mut Scope::constant(), expr))
                            .collect::<Result<_>>()
                    })
                    .collect::<Result<_>>()?,
            },

            ast::Statement::Update { table, set, r#where } => {
                let scope = &mut Scope::from_table(self.catalog.must_read_table(&table)?)?;
                Node::Update {
                    table: table.clone(),
                    source: Box::new(Node::Scan {
                        table,
                        alias: None,
                        filter: r#where.map(|e| self.build_expression(scope, e)).transpose()?,
                    }),
                    expressions: set
                        .into_iter()
                        .map(|(c, e)| {
                            Ok((
                                scope.resolve(None, &c)?,
                                Some(c),
                                self.build_expression(scope, e)?,
                            ))
                        })
                        .collect::<Result<_>>()?,
                }
            }

            // Queries.
            ast::Statement::Select {
                mut select,
                from,
                r#where,
                mut having,
                mut order,
                ..
            } => {
                let scope = &mut Scope::new();

                // Build FROM clause.
                let mut node = if !from.is_empty() {
                    self.build_from_clause(scope, from)?
                } else if select.is_empty() {
                    return Err(Error::Value("Can't select * without a table".into()));
                } else {
                    Node::Nothing
                };

                // Build WHERE clause.
                if let Some(expr) = r#where {
                    node = Node::Filter {
                        source: Box::new(node),
                        predicate: self.build_expression(scope, expr)?,
                    };
                };

                // Build SELECT clause.
                let mut hidden = 0;
                if !select.is_empty() {
                    // Inject hidden SELECT columns for fields and aggregates used in ORDER BY and
                    // HAVING expressions but not present in existing SELECT output. These will be
                    // removed again by a later projection.
                    if let Some(ref mut expr) = having {
                        hidden += self.inject_hidden(expr, &mut select)?;
                    }
                    for (expr, _) in order.iter_mut() {
                        hidden += self.inject_hidden(expr, &mut select)?;
                    }

                    // Build the remaining non-aggregate projection.
                    let expressions: Vec<(Expression, Option<String>)> = select
                        .into_iter()
                        .map(|(e, l)| Ok((self.build_expression(scope, e)?, l)))
                        .collect::<Result<_>>()?;
                    scope.project(&expressions)?;
                    node = Node::Projection { source: Box::new(node), expressions };
                };

                // Remove any hidden columns.
                if hidden > 0 {
                    node = Node::Projection {
                        source: Box::new(node),
                        expressions: (0..(scope.len() - hidden))
                            .map(|i| (Expression::Field(i, None), None))
                            .collect(),
                    }
                }

                node
            }
        })
    }

    fn build_from_clause(&self, scope: &mut Scope, from: Vec<ast::FromItem>) -> Result<Node> {
        let mut items = from.into_iter();
        let node = match items.next() {
            Some(item) => self.build_from_item(scope, item)?,
            None => return Err(Error::Value("No from items given".into())),
        };
        Ok(node)
    }

    fn build_from_item(&self, scope: &mut Scope, item: ast::FromItem) -> Result<Node> {
        Ok(match item {
            ast::FromItem::Table { name, alias } => {
                scope.add_table(
                    alias.clone().unwrap_or_else(|| name.clone()),
                    self.catalog.must_read_table(&name)?,
                )?;
                Node::Scan { table: name, alias, filter: None }
            }
            _ => unimplemented!()
        })
    }

    fn inject_hidden(
        &self,
        expr: &mut ast::Expression,
        select: &mut Vec<(ast::Expression, Option<String>)>,
    ) -> Result<usize> {
        // Replace any identical expressions or label references with column references.
        for (i, (sexpr, label)) in select.iter().enumerate() {
            if expr == sexpr {
                *expr = ast::Expression::Column(i);
                continue;
            }
            if let Some(label) = label {
                expr.transform_mut(
                    &mut |e| match e {
                        ast::Expression::Field(None, ref l) if l == label => {
                            Ok(ast::Expression::Column(i))
                        }
                        e => Ok(e),
                    },
                    &mut Ok,
                )?;
            }
        }
        // Any remaining aggregate functions and field references must be extracted as hidden
        // columns.
        let mut hidden = 0;
        expr.transform_mut(
            &mut |e| match &e {
                ast::Expression::Function(_, _) => unimplemented!(),
                ast::Expression::Field(_, _) => {
                    select.push((e, None));
                    hidden += 1;
                    Ok(ast::Expression::Column(select.len() - 1))
                }
                _ => Ok(e),
            },
            &mut Ok,
        )?;
        Ok(hidden)
    }
    
    /// Builds an expression from an AST expression
    #[allow(clippy::only_used_in_recursion)]
    fn build_expression(&self, scope: &mut Scope, expr: ast::Expression) -> Result<Expression> {
        use super::Expression::*;
        Ok(match expr {
            ast::Expression::Literal(l) => Constant(match l {
                ast::Literal::Null => Value::Null,
                ast::Literal::Boolean(b) => Value::Boolean(b),
                ast::Literal::Integer(i) => Value::Integer(i),
                ast::Literal::Float(f) => Value::Float(f),
                ast::Literal::String(s) => Value::String(s),
            }),
            ast::Expression::Column(i) => Field(i, scope.get_label(i)?),
            ast::Expression::Field(table, name) => {
                Field(scope.resolve(table.as_deref(), &name)?, Some((table, name)))
            }
            ast::Expression::Function(name, _) => {
                return Err(Error::Value(format!("Unknown function {}", name,)))
            }
            ast::Expression::Operation(op) => match op {
                // Logical operators
                ast::Operation::And(lhs, rhs) => And(
                    self.build_expression(scope, *lhs)?.into(),
                    self.build_expression(scope, *rhs)?.into(),
                ),
                ast::Operation::Not(expr) => Not(self.build_expression(scope, *expr)?.into()),
                ast::Operation::Or(lhs, rhs) => Or(
                    self.build_expression(scope, *lhs)?.into(),
                    self.build_expression(scope, *rhs)?.into(),
                ),

                // Comparison operators
                ast::Operation::Equal(lhs, rhs) => Equal(
                    self.build_expression(scope, *lhs)?.into(),
                    self.build_expression(scope, *rhs)?.into(),
                ),
                ast::Operation::GreaterThan(lhs, rhs) => GreaterThan(
                    self.build_expression(scope, *lhs)?.into(),
                    self.build_expression(scope, *rhs)?.into(),
                ),
                ast::Operation::GreaterThanOrEqual(lhs, rhs) => Or(
                    GreaterThan(
                        self.build_expression(scope, *lhs.clone())?.into(),
                        self.build_expression(scope, *rhs.clone())?.into(),
                    )
                    .into(),
                    Equal(
                        self.build_expression(scope, *lhs)?.into(),
                        self.build_expression(scope, *rhs)?.into(),
                    )
                    .into(),
                ),
                ast::Operation::IsNull(expr) => IsNull(self.build_expression(scope, *expr)?.into()),
                ast::Operation::LessThan(lhs, rhs) => LessThan(
                    self.build_expression(scope, *lhs)?.into(),
                    self.build_expression(scope, *rhs)?.into(),
                ),
                ast::Operation::LessThanOrEqual(lhs, rhs) => Or(
                    LessThan(
                        self.build_expression(scope, *lhs.clone())?.into(),
                        self.build_expression(scope, *rhs.clone())?.into(),
                    )
                    .into(),
                    Equal(
                        self.build_expression(scope, *lhs)?.into(),
                        self.build_expression(scope, *rhs)?.into(),
                    )
                    .into(),
                ),
                ast::Operation::Like(_, _) => unimplemented!(),
                ast::Operation::NotEqual(lhs, rhs) => Not(Equal(
                    self.build_expression(scope, *lhs)?.into(),
                    self.build_expression(scope, *rhs)?.into(),
                )
                .into()),

                // Mathematical operators
                ast::Operation::Assert(expr) => Assert(self.build_expression(scope, *expr)?.into()),
                ast::Operation::Add(lhs, rhs) => Add(
                    self.build_expression(scope, *lhs)?.into(),
                    self.build_expression(scope, *rhs)?.into(),
                ),
                ast::Operation::Divide(lhs, rhs) => Divide(
                    self.build_expression(scope, *lhs)?.into(),
                    self.build_expression(scope, *rhs)?.into(),
                ),
                ast::Operation::Exponentiate(lhs, rhs) => Exponentiate(
                    self.build_expression(scope, *lhs)?.into(),
                    self.build_expression(scope, *rhs)?.into(),
                ),
                ast::Operation::Factorial(expr) => {
                    Factorial(self.build_expression(scope, *expr)?.into())
                }
                ast::Operation::Modulo(lhs, rhs) => Modulo(
                    self.build_expression(scope, *lhs)?.into(),
                    self.build_expression(scope, *rhs)?.into(),
                ),
                ast::Operation::Multiply(lhs, rhs) => Multiply(
                    self.build_expression(scope, *lhs)?.into(),
                    self.build_expression(scope, *rhs)?.into(),
                ),
                ast::Operation::Negate(expr) => Negate(self.build_expression(scope, *expr)?.into()),
                ast::Operation::Subtract(lhs, rhs) => Subtract(
                    self.build_expression(scope, *lhs)?.into(),
                    self.build_expression(scope, *rhs)?.into(),
                ),
            },
        })
    }

    /// Builds and evaluates a constant AST expression.
    fn evaluate_constant(&self, expr: ast::Expression) -> Result<Value> {
        self.build_expression(&mut Scope::constant(), expr)?.evaluate(None)
    }
}

/// Manages names available to expressions and executors, and maps them onto columns/fields.
#[derive(Clone, Debug)]
pub struct Scope {
    // If true, the scope is constant and cannot contain any variables.
    constant: bool,
    // Currently visible tables, by query name (i.e. alias or actual name).
    tables: HashMap<String, Table>,
    // Column labels, if any (qualified by table name when available)
    columns: Vec<(Option<String>, Option<String>)>,
    // Qualified names to column indexes.
    qualified: HashMap<(String, String), usize>,
    // Unqualified names to column indexes, if unique.
    unqualified: HashMap<String, usize>,
    // Unqialified ambiguous names.
    ambiguous: HashSet<String>,
}

impl Scope {
    /// Creates a new, empty scope.
    fn new() -> Self {
        Self {
            constant: false,
            tables: HashMap::new(),
            columns: Vec::new(),
            qualified: HashMap::new(),
            unqualified: HashMap::new(),
            ambiguous: HashSet::new(),
        }
    }

    /// Creates a constant scope.
    fn constant() -> Self {
        let mut scope = Self::new();
        scope.constant = true;
        scope
    }

    /// Creates a scope from a table.
    fn from_table(table: Table) -> Result<Self> {
        let mut scope = Self::new();
        scope.add_table(table.name.clone(), table)?;
        Ok(scope)
    }

    /// Adds a column to the scope.
    #[allow(clippy::map_entry)]
    fn add_column(&mut self, table: Option<String>, label: Option<String>) {
        if let Some(l) = label.clone() {
            if let Some(t) = table.clone() {
                self.qualified.insert((t, l.clone()), self.columns.len());
            }
            if !self.ambiguous.contains(&l) {
                if !self.unqualified.contains_key(&l) {
                    self.unqualified.insert(l, self.columns.len());
                } else {
                    self.unqualified.remove(&l);
                    self.ambiguous.insert(l);
                }
            }
        }
        self.columns.push((table, label));
    }

    /// Adds a table to the scope.
    fn add_table(&mut self, label: String, table: Table) -> Result<()> {
        if self.constant {
            return Err(Error::Internal("Can't modify constant scope".into()));
        }
        if self.tables.contains_key(&label) {
            return Err(Error::Value(format!("Duplicate table name {}", label)));
        }
        for column in &table.columns {
            self.add_column(Some(label.clone()), Some(column.name.clone()));
        }
        self.tables.insert(label, table);
        Ok(())
    }

    /// Fetches a column from the scope by index.
    fn get_column(&self, index: usize) -> Result<(Option<String>, Option<String>)> {
        if self.constant {
            return Err(Error::Value(format!(
                "Expression must be constant, found column {}",
                index
            )));
        }
        self.columns
            .get(index)
            .cloned()
            .ok_or_else(|| Error::Value(format!("Column index {} not found", index)))
    }

    /// Fetches a column label by index, if any.
    fn get_label(&self, index: usize) -> Result<Option<(Option<String>, String)>> {
        Ok(match self.get_column(index)? {
            (table, Some(name)) => Some((table, name)),
            _ => None,
        })
    }

    /// Merges two scopes, by appending the given scope to self.
    fn merge(&mut self, scope: Scope) -> Result<()> {
        if self.constant {
            return Err(Error::Internal("Can't modify constant scope".into()));
        }
        for (label, table) in scope.tables {
            if self.tables.contains_key(&label) {
                return Err(Error::Value(format!("Duplicate table name {}", label)));
            }
            self.tables.insert(label, table);
        }
        for (table, label) in scope.columns {
            self.add_column(table, label);
        }
        Ok(())
    }

    /// Resolves a name, optionally qualified by a table name.
    fn resolve(&self, table: Option<&str>, name: &str) -> Result<usize> {
        if self.constant {
            return Err(Error::Value(format!(
                "Expression must be constant, found field {}",
                if let Some(table) = table { format!("{}.{}", table, name) } else { name.into() }
            )));
        }
        if let Some(table) = table {
            if !self.tables.contains_key(table) {
                return Err(Error::Value(format!("Unknown table {}", table)));
            }
            self.qualified
                .get(&(table.into(), name.into()))
                .copied()
                .ok_or_else(|| Error::Value(format!("Unknown field {}.{}", table, name)))
        } else if self.ambiguous.contains(name) {
            Err(Error::Value(format!("Ambiguous field {}", name)))
        } else {
            self.unqualified
                .get(name)
                .copied()
                .ok_or_else(|| Error::Value(format!("Unknown field {}", name)))
        }
    }

    /// Number of columns in the current scope.
    fn len(&self) -> usize {
        self.columns.len()
    }

    /// Projects the scope. This takes a set of expressions and labels in the current scope,
    /// and returns a new scope for the projection.
    fn project(&mut self, projection: &[(Expression, Option<String>)]) -> Result<()> {
        if self.constant {
            return Err(Error::Internal("Can't modify constant scope".into()));
        }
        let mut new = Self::new();
        new.tables = self.tables.clone();
        for (expr, label) in projection {
            match (expr, label) {
                (_, Some(label)) => new.add_column(None, Some(label.clone())),
                (Expression::Field(_, Some((Some(table), name))), _) => {
                    new.add_column(Some(table.clone()), Some(name.clone()))
                }
                (Expression::Field(_, Some((None, name))), _) => {
                    if let Some(i) = self.unqualified.get(name) {
                        let (table, name) = self.columns[*i].clone();
                        new.add_column(table, name);
                    }
                }
                (Expression::Field(i, None), _) => {
                    let (table, label) = self.columns.get(*i).cloned().unwrap_or((None, None));
                    new.add_column(table, label)
                }
                _ => new.add_column(None, None),
            }
        }
        *self = new;
        Ok(())
    }
}
