use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct ScriptParser;

#[derive(Debug, Clone)]
pub struct Script {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    StmtLabel(String),
    StmtWait(String),
    StmtEcho(String),
    StmtIf(StmtIf),
    StmtWhile(StmtWhile),
    StmtFor(StmtFor),
    StmtCall(StmtCall),
}

#[derive(Debug, Clone)]
pub struct StmtIf {
    pub condition: ExprInt,
    pub commands: Vec<StmtCall>,
}

#[derive(Debug, Clone)]
pub struct StmtWhile {
    pub condition: ExprInt,
    pub commands: Vec<StmtCall>,
}

#[derive(Debug, Clone)]
pub struct StmtFor {
    pub init: StmtCall,
    pub condition: ExprInt,
    pub increment: StmtCall,
    pub commands: Vec<StmtCall>,
}

#[derive(Debug, Clone)]
pub struct StmtCall {
    pub identifier: String,
    pub params: Vec<Value>,
}

#[derive(Debug, Clone)]
pub enum Value {
    Label(String),
    Keyword(String),
    Int(ExprInt),
    Str(ExprStr),
}

#[derive(Debug, Clone)]
pub enum ExprInt {
    Number(i64),
    VarInt(String),
    Operation(Box<ExprInt>, String, Box<ExprInt>),
}

#[derive(Debug, Clone)]
pub enum ExprStr {
    String(String),
    VarStr(String),
}

fn parse_str_expr(pair: Pair<Rule>) -> ExprStr {
    match pair.as_rule() {
        Rule::const_string => ExprStr::String(pair.as_str().to_string()),
        Rule::var_str => ExprStr::VarStr(pair.as_str().to_string()),
        _ => {
            println!("{:#?}", pair.as_rule());
            println!("{:}", pair.as_str());
            unimplemented!()
        }
    }
}

fn parse_int_expr(pair: Pair<Rule>) -> ExprInt {
    match pair.as_rule() {
        Rule::const_int => ExprInt::Number(pair.as_str().parse().unwrap()),
        Rule::var_int => ExprInt::VarInt(pair.as_str().to_string()),
        Rule::expr_int => {
            let mut iter = pair.into_inner();
            let left = parse_int_expr(iter.next().unwrap());
            let op = iter.next().unwrap().as_str().to_string();
            let right = parse_int_expr(iter.next().unwrap());
            ExprInt::Operation(Box::new(left), op, Box::new(right))
        }
        _ => {
            println!("{:#?}", pair.as_rule());
            println!("{:}", pair.as_str());
            unimplemented!()
        }
    }
}

fn parse_value(pair: Pair<Rule>) -> Value {
    match pair.as_rule() {
        Rule::color => Value::Str(ExprStr::String(pair.as_span().as_str().to_string())),
        Rule::keyword => Value::Keyword(pair.as_span().as_str().to_string()),
        Rule::label => Value::Label(pair.as_span().as_str().to_string()),
        Rule::expr_str => Value::Str(parse_str_expr(pair.into_inner().next().unwrap())),
        Rule::expr_int => Value::Int(parse_int_expr(pair.into_inner().next().unwrap())),
        _ => {
            println!("{:#?}", pair.as_rule());
            println!("{:}", pair.as_str());
            unimplemented!()
        }
    }
}

fn parse_call(pair: Pair<Rule>) -> StmtCall {
    match pair.as_rule() {
        Rule::stmt_call => {
            let mut identifier = String::new();
            let mut params = Vec::new();
            for inner_pair in pair.into_inner() {
                match inner_pair.as_rule() {
                    Rule::identifier => identifier = inner_pair.as_span().as_str().to_string(),
                    Rule::params => {
                        params = inner_pair
                            .into_inner()
                            .map(|value_pair| parse_value(value_pair))
                            .collect()
                    }
                    _ => {}
                };
            }
            StmtCall { identifier, params }
        }
        _ => {
            println!("{:#?}", pair.as_rule());
            println!("{:}", pair.as_str());
            unimplemented!()
        }
    }
}

fn parse_if(pair: Pair<Rule>) -> StmtIf {
    match pair.as_rule() {
        Rule::stmt_if => {
            let mut condition = ExprInt::Number(0);
            let mut commands = Vec::new();
            for inner_pair in pair.into_inner() {
                match inner_pair.as_rule() {
                    Rule::expr_int => condition = parse_int_expr(inner_pair),
                    Rule::stmt_call => commands.push(parse_call(inner_pair)),
                    _ => {}
                }
            }
            StmtIf {
                condition,
                commands,
            }
        }
        _ => {
            println!("{:#?}", pair.as_rule());
            println!("{:}", pair.as_str());
            unimplemented!()
        }
    }
}

fn parse_while(pair: Pair<Rule>) -> StmtWhile {
    match pair.as_rule() {
        Rule::stmt_while => {
            let mut condition = ExprInt::Number(0);
            let mut commands = Vec::new();
            for inner_pair in pair.into_inner() {
                match inner_pair.as_rule() {
                    Rule::expr_int => condition = parse_int_expr(inner_pair),
                    Rule::stmt_call => commands.push(parse_call(inner_pair)),
                    _ => {}
                }
            }
            StmtWhile {
                condition,
                commands,
            }
        }
        _ => {
            println!("{:#?}", pair.as_rule());
            println!("{:}", pair.as_str());
            unimplemented!()
        }
    }
}

fn parse_for(pair: Pair<Rule>) -> StmtFor {
    match pair.as_rule() {
        Rule::stmt_for => {
            let mut init = StmtCall {
                identifier: String::new(),
                params: Vec::new(),
            };
            let mut condition = ExprInt::Number(0);
            let mut increment = StmtCall {
                identifier: String::new(),
                params: Vec::new(),
            };
            let mut commands = Vec::new();
            for inner_pair in pair.into_inner() {
                match inner_pair.as_rule() {
                    Rule::stmt_call => {
                        if init.identifier.is_empty() {
                            init = parse_call(inner_pair);
                        } else if increment.identifier.is_empty() {
                            increment = parse_call(inner_pair);
                        } else {
                            commands.push(parse_call(inner_pair));
                        }
                    }
                    Rule::expr_int => condition = parse_int_expr(inner_pair),
                    _ => {}
                }
            }
            StmtFor {
                init,
                condition,
                increment,
                commands,
            }
        }
        _ => {
            println!("{:#?}", pair.as_rule());
            println!("{:}", pair.as_str());
            unimplemented!()
        }
    }
}

fn parse_stmt(pair: Pair<Rule>) -> Stmt {
    match pair.as_rule() {
        Rule::stmt_label => Stmt::StmtLabel(pair.as_str().to_string()),
        Rule::stmt_wait => Stmt::StmtWait(pair.as_str().to_string()),
        Rule::stmt_echo => Stmt::StmtEcho(pair.as_str().to_string()),
        Rule::stmt_call => Stmt::StmtCall(parse_call(pair)),
        Rule::stmt_if => Stmt::StmtIf(parse_if(pair)),
        Rule::stmt_while => Stmt::StmtWhile(parse_while(pair)),
        Rule::stmt_for => Stmt::StmtFor(parse_for(pair)),
        _ => {
            println!("{:#?}", pair.as_rule());
            println!("{:#?}", pair.as_str());
            unreachable!()
        }
    }
}

fn parse_script(pair: Pair<Rule>) -> Script {
    match pair.as_rule() {
        Rule::script => Script {
            stmts: pair
                .into_inner()
                .filter(|i| i.as_rule() != Rule::EOI)
                .map(parse_stmt)
                .collect(),
        },
        _ => {
            println!("{:#?}", pair.as_rule());
            println!("{:#?}", pair.as_str());
            unreachable!()
        }
    }
}

pub fn parse(s: &str) -> Option<Script> {
    ScriptParser::parse(Rule::script, s)
        .ok()
        .and_then(|mut s| s.next())
        .map(parse_script)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let s = std::fs::read_to_string("./tests/1.txt").unwrap();
        let script = parse(&s).unwrap();
        println!("{:#?}", script);
    }
}
