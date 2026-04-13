use pest::Parser;
use pest::iterators::{Pair, Pairs};
use pest_derive::Parser;

use crate::states::{Def, Location, NamedStruct, Program, Ptr, Region, Step, Value};

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct StatesParser;

/// Parse kaya string, return error if parse fails
pub fn parse(content: &str) -> Result<Program, pest::error::Error<Rule>> {
    // Always start with newline at end to avoid confusing error messages at end of program
    let padded_content = format!("{}\n", content);
    let pairs = StatesParser::parse(Rule::start, &padded_content)?;
    Ok(parse_program(pairs))
}

/// Parse kaya string, return as much program as we can parse (stop parsing rather than error)
pub fn parse_partial(content: &str) -> Result<Program, pest::error::Error<Rule>> {
    let pairs = StatesParser::parse(Rule::start_partial, content)?;
    Ok(parse_program(pairs))
}

fn parse_program(pairs: Pairs<Rule>) -> Program {
    let mut steps = Vec::new();

    for pair in pairs {
        match pair.as_rule() {
            Rule::step => steps.push(parse_step(pair)),
            Rule::EOL => {}
            Rule::start | Rule::start_partial => {
                for inner in pair.into_inner() {
                    #[allow(clippy::single_match)]
                    match inner.as_rule() {
                        Rule::step => steps.push(parse_step(inner)),
                        _ => {}
                    }
                }
            }
            _ => unreachable!(),
        }
    }

    Program(steps)
}

fn parse_step(pair: Pair<Rule>) -> Step {
    let mut label = None;
    let mut locations = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::TEXT => {
                label = Some(inner.as_str().to_string());
            }
            Rule::location => {
                locations.push(parse_location(inner));
            }
            Rule::EOL => {}
            _ => {}
        }
    }

    Step {
        label: label.unwrap_or_default(),
        locations,
    }
}

fn parse_location(pair: Pair<Rule>) -> Location {
    let mut name = None;
    let mut regions = Vec::new();
    let mut definitions = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::TEXT => {
                name = Some(inner.as_str().to_string());
            }
            Rule::location_content => {
                for content in inner.into_inner() {
                    match content.as_rule() {
                        Rule::region => {
                            regions.push(parse_region(content));
                        }
                        Rule::defln_ => {
                            definitions.push(parse_defln(content));
                        }
                        _ => {}
                    }
                }
            }
            Rule::EOL => {}
            _ => {}
        }
    }

    Location {
        name: name.unwrap_or_default(),
        regions,
        definitions,
    }
}

fn parse_region(pair: Pair<Rule>) -> Region {
    let mut name = None;
    let mut definitions = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::TEXT => {
                name = Some(inner.as_str().to_string());
            }
            Rule::defln_ => {
                definitions.push(parse_defln(inner));
            }
            Rule::EOL => {}
            _ => {}
        }
    }

    Region {
        name: name.unwrap_or_default(),
        definitions,
    }
}

fn parse_defln(pair: Pair<Rule>) -> Def {
    let mut label = None;
    let mut value = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::def_ => {
                let (l, v) = parse_def(inner);
                label = Some(l);
                value = Some(v);
            }
            Rule::EOL => {}
            _ => {}
        }
    }

    Def {
        label: label.unwrap_or_default(),
        value: value.unwrap_or(Value::Invalid),
    }
}

fn parse_def(pair: Pair<Rule>) -> (String, Value) {
    let mut label = None;
    let mut value = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::label => {
                label = Some(parse_label(inner));
            }
            Rule::value => {
                value = Some(parse_value(inner));
            }
            _ => {}
        }
    }

    (label.unwrap_or_default(), value.unwrap_or(Value::Invalid))
}

fn parse_value(pair: Pair<Rule>) -> Value {
    let inner = pair.clone().into_inner().next().unwrap_or(pair);
    match inner.as_rule() {
        Rule::number => parse_number(inner),
        Rule::array_value => parse_array_value(inner),
        Rule::tuple_value => parse_tuple_value(inner),
        Rule::char_value => parse_char_value(inner),
        Rule::struct_value => parse_struct_value(inner),
        Rule::ptr_value => parse_ptr_value(inner),
        Rule::invalid_value => Value::Invalid,
        _ => Value::Invalid,
    }
}

fn parse_number(pair: Pair<Rule>) -> Value {
    let mut is_negative = false;
    let mut has_sign = false;

    let content = pair.as_str();
    let mut chars = content.chars().peekable();

    if let Some(&c) = chars.peek() {
        if c == '-' {
            is_negative = true;
            has_sign = true;
            chars.next();
        } else if c == '+' {
            has_sign = true;
            chars.next();
        }
    }

    let num_str = if has_sign {
        chars.collect::<String>()
    } else {
        content.to_string()
    };

    let val: f64 = num_str.parse().unwrap_or(0.0);
    Value::Number(if is_negative { -val } else { val })
}

fn parse_array_value(pair: Pair<Rule>) -> Value {
    let mut values = Vec::new();
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::value {
            values.push(parse_value(inner));
        }
    }
    Value::Array(values)
}

fn parse_tuple_value(pair: Pair<Rule>) -> Value {
    let mut values = Vec::new();
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::value {
            values.push(parse_value(inner));
        }
    }
    Value::Tuple(values)
}

fn parse_char_value(pair: Pair<Rule>) -> Value {
    let content = pair.as_str();
    if content.len() >= 3 {
        let ch = content.chars().nth(1).unwrap_or(' ');
        Value::Char(ch)
    } else {
        Value::Invalid
    }
}

fn parse_struct_value(pair: Pair<Rule>) -> Value {
    let mut name = None;
    let mut fields = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::label => {
                name = Some(parse_label(inner));
            }
            Rule::def_ => {
                fields.push(parse_def(inner));
            }
            _ => {}
        }
    }

    Value::Struct(NamedStruct {
        name: name.unwrap_or_default(),
        fields,
    })
}

fn parse_ptr_value(pair: Pair<Rule>) -> Value {
    let mut res = Ptr {
        name: "".into(),
        selectors: vec![],
        borrow: 0,
        help: vec![],
    };
    let mut valid = false;
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::destination {
            res = parse_destination(inner);
            valid = true;
        } else if inner.as_rule() == Rule::help {
            res.help.push(inner.as_str().to_string());
        }
    }
    if valid {
        return Value::Pointer(res);
    }
    Value::Invalid
}

fn parse_destination(pair: Pair<Rule>) -> Ptr {
    let mut name = None;
    let mut selectors = Vec::new();
    let mut borrow = 0;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::label => {
                name = Some(parse_label(inner));
            }
            Rule::DIGITS => {
                let val: u32 = inner.as_str().parse().unwrap_or(0);
                selectors.push(val);
            }
            Rule::borrow => {
                borrow = count_borrows(inner);
            }
            _ => {}
        }
    }

    Ptr {
        name: name.unwrap_or_default(),
        selectors,
        borrow,
        help: vec![],
    }
}

fn count_borrows(pair: Pair<Rule>) -> usize {
    pair.as_str().chars().filter(|&c| c == '\'').count()
}

fn parse_label(pair: Pair<Rule>) -> String {
    let content = pair.as_str();
    if content.starts_with('`') && content.ends_with('`') && content.len() >= 2 {
        content[1..content.len() - 1].to_string()
    } else if content == "(return)" {
        "(return)".to_string()
    } else {
        content.to_string()
    }
}
