use pest::Parser;
use pest_derive::*;

use crate::types::{KeyValue, Labels};

#[derive(Parser)]
#[grammar = "labels.pest"]
pub struct LabelParser;

pub fn labels_from_str(input: &str) -> Labels {
    let mut res = Vec::new();
    for pair in LabelParser::parse(Rule::labels, input).unwrap() {
        match pair.as_rule() {
            Rule::label => {
                let mut i = pair.into_inner();
                let key = i.next().unwrap().as_str();
                let value = i.next().unwrap().as_str();
                res.push(KeyValue::new(key.to_string(), value.to_string()));
                assert!(i.next().is_none());
            }
            Rule::EOI => (),
            _ => unreachable!(),
        }
    }
    res
}
