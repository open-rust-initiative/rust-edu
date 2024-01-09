use super::errors::{format_parse_error, FormattedError};
use super::types::*;
// Using tag_no_case from nom_supreme since its error is nicer
// ParserExt is mostly for adding `.context` on calls to identifier to say what kind of identifier we want
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{char, i32 as int32, multispace0, multispace1, none_of},
    combinator::{all_consuming, cut, map, opt},
    error::context,
    multi::{many0, separated_list1},
    sequence::{delimited, preceded, separated_pair, tuple},
    Finish,
};
use nom_supreme::{tag::complete::tag_no_case, ParserExt};
/// Parse a unquoted sql identifier
fn identifier(i: Span) -> ParseResult<String> {
    map(
        take_while1(|c: char| c.is_alphanumeric() || c == '_'),
        |s: Span| s.fragment().to_string(),
    )(i)
}

fn comma_sep<'a, O, F>(f: F) -> impl FnMut(Span<'a>) -> ParseResult<'a, Vec<O>>
where
    F: FnMut(Span<'a>) -> ParseResult<'a, O>,
{
    delimited(
        multispace0,
        separated_list1(tuple((multispace0, char(','), multispace0)), f),
        multispace0,
    )
}

/// Implement the parse function to more easily convert a span into a sql
/// command
pub trait Parse<'a>: Sized {
    /// Parse the given span into self
    fn parse(input: Span<'a>) -> ParseResult<'a, Self>;
    /// Helper method for tests to convert a str into a raw span and parse
    /// It's recommanded to use `parse_format_error`
    fn parse_from_raw(input: &'a str) -> ParseResult<'a, Self> {
        let i = Span::new(input);
        Self::parse(i)
    }
    /// Parse API that could return formatted Error
    /// # Usage
    /// ```
    /// match SqlQuery::parse_format_error(query) {
    ///     Ok(q) => println!("{q:?}"),
    ///     Err(e) => {
    ///         let mut s = String::new();
    ///         GraphicalReportHandler::new()
    ///             .render_report(&mut s, &e)
    ///             .unwrap();
    ///         println!("{s}");
    ///     }
    /// }
    /// ```
    fn parse_format_error(i: &'a str) -> Result<Self, FormattedError<'a>> {
        let input = Span::new(i);
        // match Self::parse(input).finish() {
        match all_consuming(Self::parse)(input).finish() {
            Ok((_, query)) => Ok(query),
            Err(e) => Err(format_parse_error(i, e)),
        }
    }
}

// parses "string | int"
impl<'a> Parse<'a> for SqlType {
    fn parse(input: Span<'a>) -> ParseResult<'a, Self> {
        // context will help give better error messages later on
        context(
            "Column Type",
            // alt will try each passed parser and return what ever succeeds
            alt((
                map(tag_no_case("string"), |_| Self::String),
                map(tag_no_case("int"), |_| Self::Int),
            )),
        )(input)
    }
}

/// parses "<colName> <colType>"
impl<'a> Parse<'a> for Column {
    fn parse(input: Span<'a>) -> ParseResult<'a, Self> {
        context(
            "Create Column",
            map(
                separated_pair(
                    identifier.context("Column Name"),
                    multispace1,
                    SqlType::parse,
                ),
                |(name, type_info)| Self { name, type_info },
            ),
        )(input)
    }
}

// parses a comma seperated list of column definitions contained in parens
fn column_definitions(input: Span<'_>) -> ParseResult<'_, Vec<Column>> {
    context(
        "Column Definitions",
        map(
            delimited(
                tuple((multispace0, char('('))),
                comma_sep(Column::parse),
                tuple((multispace0, char(')'))),
            ),
            |cols| cols,
        ),
    )(input)
}

// parses "CREATE TABLE <table name> <column defs>
impl<'a> Parse<'a> for CreateStatement {
    fn parse(input: Span<'a>) -> ParseResult<'a, Self> {
        map(
            separated_pair(
                // table name
                preceded(
                    tuple((
                        tag_no_case("create"),
                        multispace1,
                        tag_no_case("table"),
                        multispace1,
                    )),
                    cut(identifier.context("Table Name")),
                ),
                multispace1,
                // column defs
                cut(column_definitions),
            )
            .context("Create Table"),
            |(table, columns)| Self { table, columns },
        )(input)
    }
}

impl<'a> Parse<'a> for DropStatement {
    fn parse(input: Span<'a>) -> ParseResult<'a, Self> {
        map(
            // table name
            preceded(
                tuple((
                    tag_no_case("drop"),
                    multispace1,
                    tag_no_case("table"),
                    multispace1,
                )),
                cut(identifier.context("Table Name")),
            )
            .context("Drop Table"),
            |table| Self { table },
        )(input)
    }
}

/// String value in SQL statement should be wrapped with apostrophes
impl<'a> Parse<'a> for String {
    fn parse(input: Span<'a>) -> ParseResult<'a, Self> {
        map(
            delimited(char('\''), many0(none_of("\'")), char('\'')),
            |chars| String::from_iter(chars.iter()),
        )(input)
    }
}

impl<'a> Parse<'a> for SqlValue {
    fn parse(input: Span<'a>) -> ParseResult<'a, Self> {
        context(
            "Sql Value",
            alt((
                map(int32, |i| Self::Int(i)),
                map(String::parse, |s| Self::String(s)),
            )),
        )(input)
    }
}

impl<'a> Parse<'a> for RowValue {
    fn parse(input: Span<'a>) -> ParseResult<'a, Self> {
        // context will help give better error messages later on
        map(
            context(
                "Value of Insert Row",
                // alt will try each passed parser and return what ever succeeds
                delimited(
                    tuple((multispace0, char('('), multispace0)),
                    comma_sep(SqlValue::parse),
                    tuple((multispace0, char(')'), multispace0)),
                ),
            ),
            |values| Self { values },
        )(input)
    }
}

/// Column clause in insert statement
fn insert_columns<'a>(input: Span<'a>) -> ParseResult<'a, Vec<String>> {
    context(
        "Columns clause wrapped with parentheses",
        delimited(
            tuple((multispace0, char('('))),
            comma_sep(identifier),
            tuple((multispace0, char(')'))),
        ),
    )(input)
}

impl<'a> Parse<'a> for InsertStatement {
    fn parse(input: Span<'a>) -> ParseResult<'a, Self> {
        map(
            tuple((
                preceded(
                    // table name
                    tuple((
                        multispace0,
                        tag_no_case("insert"),
                        multispace1,
                        cut(tag_no_case("into")),
                        multispace1,
                    )),
                    cut(identifier.context("Table Name")),
                ),
                opt(insert_columns),
                cut(preceded(
                    tuple((multispace0, tag_no_case("values"))),
                    RowValue::parse,
                )),
            ))
            .context("Insert Rows"),
            |(table, columns, values)| Self {
                table,
                columns,
                values,
            },
        )(input)
    }
}

impl<'a> Parse<'a> for CmpOpt {
    fn parse(input: Span<'a>) -> ParseResult<'a, Self> {
        context(
            "Compare Operator",
            alt((
                // ATTENTION: 顺序很重要!!!!
                map(tag("="), |_| Self::Eq),
                map(tag("<>"), |_| Self::Ne),
                map(tag("<="), |_| Self::Le),
                map(tag("<"), |_| Self::Lt),
                map(tag(">="), |_| Self::Ge),
                map(tag(">"), |_| Self::Gt),
            )),
        )(input)
    }
}

impl<'a> WhereConstraint {
    fn parse_constrait(input: Span<'a>) -> ParseResult<'a, Self> {
        context(
            "Constrait",
            map(
                tuple((
                    multispace0,
                    identifier,
                    multispace0,
                    CmpOpt::parse,
                    multispace0,
                    cut(SqlValue::parse),
                )),
                |(_, column, _, op, _, value)| Self::Constrait(column, op, value),
            ),
        )(input)
    }

    fn parse_and(input: Span<'a>) -> ParseResult<'a, Self> {
        context(
            "Constraits combined with `and`",
            map(
                tuple((
                    multispace0,
                    Self::parse_constrait,
                    multispace1,
                    tag_no_case("and"),
                    cut(Self::parse_constraits),
                )),
                |(_, cons_l, _, _, cons_r)| Self::And(Box::new(cons_l), Box::new(cons_r)),
            ),
        )(input)
    }

    fn parse_or(input: Span<'a>) -> ParseResult<'a, Self> {
        context(
            "Constraits combined with `or`",
            map(
                tuple((
                    multispace0,
                    Self::parse_constrait,
                    multispace1,
                    tag_no_case("or"),
                    cut(Self::parse_constraits),
                )),
                |(_, cons_l, _, _, cons_r)| Self::Or(Box::new(cons_l), Box::new(cons_r)),
            ),
        )(input)
    }

    fn parse_not(input: Span<'a>) -> ParseResult<'a, Self> {
        context(
            "Not Clause",
            map(
                preceded(
                    tuple((multispace0, tag_no_case("not"))),
                    cut(Self::parse_constrait),
                ),
                |cons| Self::Not(Box::new(cons)),
            ),
        )(input)
    }

    fn parse_constraits(input: Span<'a>) -> ParseResult<'a, Self> {
        alt((Self::parse_not, Self::parse_and, Self::parse_or, cut(Self::parse_constrait)))(input)
    }
}

impl<'a> Parse<'a> for WhereConstraint {
    fn parse(input: Span<'a>) -> ParseResult<'a, Self> {
        context(
            "Where Constraint",
            preceded(
                tuple((multispace0, tag_no_case("where"))),
                // parse_constraint must be the last one in the alt list
                Self::parse_constraits,
            ),
        )(input)
    }
}

fn result_columns<'a>(input: Span<'a>) -> ParseResult<'a, Vec<String>> {
    context(
        "Result Columns",
        comma_sep(alt((identifier, map(tag("*"), |_| String::from("*"))))),
    )(input)
}

impl<'a> Parse<'a> for SelectStatement {
    fn parse(input: Span<'a>) -> ParseResult<'a, Self> {
        context(
            "Select Statement",
            map(
                preceded(
                    tuple((multispace0, tag_no_case("select"), multispace0)),
                    cut(tuple((
                        result_columns,
                        multispace0,
                        tag_no_case("from"),
                        multispace0,
                        identifier,
                        opt(WhereConstraint::parse),
                    ))),
                ),
                |(columns, _, _, _, table, constraints)| Self {
                    table,
                    columns,
                    constraints,
                },
            ),
        )(input)
    }
}

impl<'a> Parse<'a> for DeleteStatement {
    fn parse(input: Span<'a>) -> ParseResult<'a, Self> {
        context(
            "Delete Statement",
            map(
                preceded(
                    tuple((
                        multispace0,
                        tag_no_case("delete"),
                        multispace1,
                        tag_no_case("from"),
                        multispace1,
                    )),
                    cut(tuple((identifier, opt(WhereConstraint::parse)))),
                ),
                |(table, constraints)| Self { table, constraints },
            ),
        )(input)
    }
}

impl<'a> Parse<'a> for SetItem {
    fn parse(input: Span<'a>) -> ParseResult<'a, Self> {
        context(
            "Set Item",
            map(
                separated_pair(
                    identifier,
                    tuple((multispace0, char('='), multispace0)),
                    SqlValue::parse,
                ),
                |(column, value)| Self { column, value },
            ),
        )(input)
    }
}

impl<'a> Parse<'a> for UpdateStatement {
    fn parse(input: Span<'a>) -> ParseResult<'a, Self> {
        context(
            "Update Statement",
            map(
                preceded(
                    tuple((multispace0, tag_no_case("update"), multispace1)),
                    cut(tuple((
                        identifier,
                        multispace1,
                        cut(tag_no_case("set")),
                        multispace0,
                        comma_sep(SetItem::parse),
                        multispace0,
                        opt(WhereConstraint::parse),
                    ))),
                ),
                |(table, _, _, _, sets, _, constraints)| Self {
                    table,
                    sets,
                    constraints,
                },
            ),
        )(input)
    }
}

impl<'a> Parse<'a> for SqlQuery {
    fn parse(input: Span<'a>) -> ParseResult<'a, Self> {
        let (rest, (query, _, _, _)) = tuple((
            alt((
                // this feels ripe for a derive macro but another time....
                map(SelectStatement::parse, SqlQuery::Select),
                map(InsertStatement::parse, SqlQuery::Insert),
                map(CreateStatement::parse, SqlQuery::Create),
                map(DeleteStatement::parse, SqlQuery::Delete),
                map(DropStatement::parse, SqlQuery::Drop),
                map(UpdateStatement::parse, SqlQuery::Update),
            )),
            multispace0,
            char(';'),
            multispace0,
        ))(input)?;
        Ok((rest, query))
    }
}

#[cfg(test)]
mod test_create_stmt {
    use super::*;
    #[test]
    fn test_create_stmt1() {
        let expected = CreateStatement {
            table: "foo".into(),
            columns: vec![
                Column {
                    name: "col1".into(),
                    type_info: SqlType::Int,
                },
                Column {
                    name: "col2".into(),
                    type_info: SqlType::String,
                },
                Column {
                    name: "col3".into(),
                    type_info: SqlType::String,
                },
            ],
        };
        assert_eq!(
            CreateStatement::parse_from_raw(
                "CREATE TABLE foo (col1 int, col2 string, col3 string)"
            )
            .unwrap()
            .1,
            expected
        )
    }
}

#[cfg(test)]
mod test_drop_stmt {
    use super::*;
    #[test]
    fn test_drop_stmt1() {
        let expected = DropStatement {
            table: "foo".into(),
        };
        assert_eq!(
            DropStatement::parse_from_raw("DROP TABLE foo").unwrap().1,
            expected
        )
    }
}

#[cfg(test)]
mod test_insert_stmt {
    use super::*;
    #[test]
    fn test_insert_stmt() {
        let expected = InsertStatement {
            table: String::from("foo"),
            columns: None,
            values: RowValue {
                values: vec![
                    SqlValue::String(String::from("abc")),
                    SqlValue::Int(123),
                    SqlValue::String(String::from("def")),
                ],
            },
        };
        let parse_result =
            InsertStatement::parse_from_raw("INSERT INTO foo VALUES ('abc', 123, 'def')")
                .unwrap()
                .1;
        assert_eq!(parse_result, expected)
    }

    #[test]
    fn test_insert_stmt2() {
        let expected = InsertStatement {
            table: String::from("foo"),
            columns: Some(vec![
                String::from("name"),
                String::from("id"),
                String::from("value"),
            ]),
            values: RowValue {
                values: vec![
                    SqlValue::String(String::from("abc")),
                    SqlValue::Int(123),
                    SqlValue::String(String::from("def")),
                ],
            },
        };
        let parse_result = InsertStatement::parse_from_raw(
            "INSERT INTO foo (name, id, value) VALUES ('abc', 123, 'def')",
        )
        .unwrap()
        .1;
        assert_eq!(parse_result, expected)
    }
}

#[cfg(test)]
mod test_select_stmt {
    use super::*;
    #[test]
    fn test_select_stmt1() {
        let parse_result = SelectStatement::parse_from_raw(
            "SELECT abc, value, bar FROM foo WHERE bar = 123 AND abc >= 'def'",
        );
        match parse_result {
            Ok(q) => println!("{q:?}"),
            Err(e) => eprintln!("{e:?}"),
        }
    }

    #[test]
    fn test_select_stmt2() {
        let expected = SelectStatement {
            table: String::from("foo"),
            columns: vec![
                String::from("abc"),
                String::from("value"),
                String::from("*"),
            ],
            constraints: Some(WhereConstraint::And(
                Box::new(WhereConstraint::Constrait(
                    String::from("bar"),
                    CmpOpt::Eq,
                    SqlValue::Int(123),
                )),
                Box::new(WhereConstraint::Constrait(
                    String::from("abc"),
                    CmpOpt::Le,
                    SqlValue::String(String::from("def")),
                )),
            )),
        };
        let parse_result = SelectStatement::parse_from_raw(
            "SELECT abc, value, * from foo WHERE bar = 123 AND abc <= 'def'",
        )
        .unwrap()
        .1;
        assert_eq!(parse_result, expected)
    }
}

#[cfg(test)]
mod test_delete_stmt {
    use super::*;
    #[test]
    fn test_select_stmt2() {
        let expected = DeleteStatement {
            table: String::from("foo"),
            constraints: Some(WhereConstraint::And(
                Box::new(WhereConstraint::Constrait(
                    String::from("bar"),
                    CmpOpt::Eq,
                    SqlValue::Int(123),
                )),
                Box::new(WhereConstraint::Constrait(
                    String::from("abc"),
                    CmpOpt::Le,
                    SqlValue::String(String::from("def")),
                )),
            )),
        };
        let parse_result =
            DeleteStatement::parse_from_raw("DELETE FROM foo WHERE bar = 123 AND abc <= 'def'")
                .unwrap()
                .1;
        assert_eq!(parse_result, expected)
    }
}

#[cfg(test)]
mod test_update_stmt {
    use super::*;
    #[test]
    fn test_update_stmt1() {
        let parse_result = UpdateStatement::parse_from_raw(
            "UPDATE foo SET abc=123, def='xyz' WHERE abc < 123 AND def = 'def'",
        );
        match parse_result {
            Ok(q) => println!("{q:?}"),
            Err(e) => eprintln!("{e:?}"),
        }
    }

    #[test]
    fn test_update_stmt2() {
        let expected = UpdateStatement {
            table: "foo".into(),
            sets: vec![
                SetItem {
                    column: "abc".into(),
                    value: SqlValue::Int(123),
                },
                SetItem {
                    column: "def".into(),
                    value: SqlValue::String("xyz".into()),
                },
            ],
            constraints: Some(WhereConstraint::And(
                Box::new(WhereConstraint::Constrait(
                    String::from("abc"),
                    CmpOpt::Lt,
                    SqlValue::Int(123),
                )),
                Box::new(WhereConstraint::Constrait(
                    String::from("def"),
                    CmpOpt::Eq,
                    SqlValue::String(String::from("def")),
                )),
            )),
        };

        let parse_result = UpdateStatement::parse_from_raw(
            "UPDATE foo SET abc=123, def='xyz' WHERE abc < 123 AND def = 'def'",
        )
        .unwrap()
        .1;
        assert_eq!(parse_result, expected)
    }
}

#[cfg(test)]
mod test_query {
    use super::*;
    use miette::GraphicalReportHandler;
    fn print_error(query: &str) {
        match SqlQuery::parse_format_error(query) {
            Ok(q) => println!("{q:?}"),
            Err(e) => {
                let mut s = String::new();
                GraphicalReportHandler::new()
                    .render_report(&mut s, &e)
                    .unwrap();
                println!("{s}");
            }
        }
    }

    #[test]
    fn test_print_error1() {
        let query = "SELECT abc, value, * from foo WHERE bar = 123 AND abc ";
        print_error(query);
    }

    #[test]
    fn test_print_error2() {
        let query = "SELECT abc, value, * foo from WHERE bar = 123 AND abc <= 'def';";
        print_error(query);
    }

    #[test]
    fn test_print_error3() {
        let query = "DROP TABLE foo,";
        print_error(query);
    }

    #[test]
    fn test_select() {
        let expected = SelectStatement {
            table: String::from("foo"),
            columns: vec![
                String::from("abc"),
                String::from("value"),
                String::from("*"),
            ],
            constraints: Some(WhereConstraint::And(
                Box::new(WhereConstraint::Constrait(
                    String::from("bar"),
                    CmpOpt::Eq,
                    SqlValue::Int(123),
                )),
                Box::new(WhereConstraint::Constrait(
                    String::from("abc"),
                    CmpOpt::Le,
                    SqlValue::String(String::from("def")),
                )),
            )),
        };
        assert_eq!(
            SqlQuery::parse_from_raw(
                "SELECT abc, value, * from foo WHERE bar = 123 AND abc <= 'def';"
            )
            .unwrap()
            .1,
            SqlQuery::Select(expected)
        )
    }
}
