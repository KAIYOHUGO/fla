use std::fmt::Display;

use pest::{error::Error, Parser};
use pest_derive::Parser;

pub type Root<'a> = Vec<Item<'a>>;

#[derive(Parser)]
#[grammar = "fla.pest"] // relative to src
pub struct PestParser {}

#[derive(Debug, Clone)]
pub enum Item<'s> {
    Pair(Pair<'s>),
}

#[derive(Debug, Clone)]
pub struct Pair<'s> {
    pub key: Vec<Key<'s>>,
    pub value: Vec<Value<'s>>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub enum Key<'s> {
    Text(&'s str),
    Cloze(&'s str),
}

#[derive(Debug, Clone)]
pub enum Value<'s> {
    Node(Node<'s>),
    Text(&'s str),
}

#[derive(Debug, Clone)]
pub struct Node<'s> {
    pub speech: Speech,
    pub text: &'s str,
}

#[derive(Debug, Clone)]
pub enum Speech {
    N,
    V,
    O,
    Adj,
    Adv,
    Prep,
    Pron,
}

impl Display for Speech {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Speech::N => write!(f, "n"),
            Speech::V => write!(f, "v"),
            Speech::O => write!(f, "o"),
            Speech::Adj => write!(f, "adj"),
            Speech::Adv => write!(f, "adv"),
            Speech::Prep => write!(f, "prep"),
            Speech::Pron => write!(f, "pron"),
        }
    }
}

pub fn parse(s: &str) -> Result<Root, Error<Rule>> {
    let mut raw_ast = PestParser::parse(Rule::root, s)?;
    // dbg!(&raw_ast);
    let r = raw_ast
        .next()
        .unwrap()
        .into_inner()
        .map(|item| match item.as_rule() {
            Rule::pair => {
                let mut inner = item.into_inner();
                let key = {
                    inner
                        .next()
                        .unwrap()
                        .into_inner()
                        .filter_map(|key| match key.as_rule() {
                            Rule::key_text => Some(Key::Text(clean_text(key.as_str())?)),
                            Rule::cloze => Some(Key::Cloze(clean_text(key.as_str())?)),
                            _ => unreachable!(),
                        })
                        .collect()
                };

                let value = {
                    let mut r = vec![];
                    for value in inner.next().unwrap().into_inner() {
                        match value.as_rule() {
                            Rule::node => {
                                let mut inner = value.into_inner();
                                let speech = match inner.next().unwrap().as_str() {
                                    "n" => Speech::N,
                                    "v" => Speech::V,
                                    "o" => Speech::O,
                                    "adj" => Speech::Adj,
                                    "adv" => Speech::Adv,
                                    "prep" => Speech::Prep,
                                    "pron" => Speech::Pron,
                                    _ => unreachable!(),
                                };
                                let text = inner.next().unwrap().as_str().trim();
                                r.push(Value::Node(Node { speech, text }));
                            }
                            Rule::value_text => {
                                let s = value.as_str().trim();
                                if !s.is_empty() {
                                    r.push(Value::Text(s));
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                    r
                };
                Item::Pair(Pair { key, value })
            }
            _ => unreachable!(),
        })
        .collect();
    Ok(r)
}

fn clean_text(value: &str) -> Option<&str> {
    let s = value.trim();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}
