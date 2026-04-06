use std::{iter::Peekable, rc::Rc};

use crate::{
    knot_core::DirList,
    reidemiester::{knot_commute, knot_epsilon, knot_stab, knot_switch},
};
use regex::Regex;
#[derive(Clone, Debug)]
struct ListSearchType {
    pub contains: Box<Vec<SearchType>>,
    curr: usize,
}

#[derive(Clone, Debug)]
struct UnionSearchType {
    pub contains: Box<Vec<fn(&DirList) -> Vec<DirList>>>,
}

impl ListSearchType {
    pub fn next(&mut self) -> Option<Box<dyn Fn(&DirList) -> Vec<DirList> + Send + Sync>> {
        if self.curr < self.contains.len() {
            let mut result = self.contains[self.curr].clone();
            if let Some(some) = result.next() {
                Some(some)
            } else {
                self.curr += 1;
                self.next()
            }
        } else {
            self.curr = 0;
            None
        }
    }
}

impl UnionSearchType {
    pub fn next(
        &mut self,
    ) -> Option<Box<dyn Fn(&DirList) -> Vec<DirList> + Send + Sync>> {
        if self.contains.is_empty() {
            return None;
        }

        let funcs = (*self.contains).clone();

        Some(Box::new(move |input: &DirList| {
            let mut out = Vec::new();
            for f in &funcs {
                out.extend(f(input));
            }
            out
        }))
    }
}

#[derive(Clone, Debug)]
struct RepeatSearchType {
    pub count: i32,
    pub contains: Box<SearchType>,
    curr: i32,
}

impl RepeatSearchType {
    pub fn next(&mut self) -> Option<Box<dyn Fn(&DirList) -> Vec<DirList> + Send + Sync>> {
        if self.curr < self.count {
            if let Some(some) = self.contains.next() {
                Some(some)
            } else {
                self.curr += 1;
                self.next()
            }
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
enum SearchType {
    Function(fn(&DirList) -> Vec<DirList>),
    Repeat(RepeatSearchType),
    List(ListSearchType),
    Union(UnionSearchType),
}

impl SearchType {
    pub fn next(&mut self) -> Option<Box<dyn Fn(&DirList) -> Vec<DirList> + Send + Sync>> {
        match self {
            SearchType::Function(f) => Some(Box::new(*f)),
            SearchType::Repeat(f) => f.next(),
            SearchType::List(f) => f.next(),
            SearchType::Union(f) => f.next(),
        }
    }

    pub fn wait_for_next(&mut self) -> Box<dyn Fn(&DirList) -> Vec<DirList> + Send + Sync> {
        loop {
            if let Some(result) = self.next() { 
                return result;
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct KnotFinder(SearchType);

impl KnotFinder {
    pub fn next(&mut self) -> Option<Box<dyn Fn(&DirList) -> Vec<DirList> + Send + Sync>> {
        self.0.next()
    }

    pub fn build(depth: i32, function: fn(&DirList) -> Vec<DirList>) -> KnotFinder {
        KnotFinder(SearchType::Repeat(RepeatSearchType {
            contains: Box::new(SearchType::Function(function)),
            count: depth,
            curr: 0,
        }))
    }
}

pub fn read_to_knot_finder(filename: String) -> KnotFinder {
    let binding = &std::fs::read(filename).unwrap();
    let text = str::from_utf8(binding).unwrap();

    // tokenize:
    let re = Regex::new(
        r#"(?x)
        \d+                             |
        [[[:alpha:]]\_]+                |
        '.+?'                           |
        ".*"+?                          |
        [=+*/%&|<>!?^~\#\-]+   |
        [\(\)\[\]\{\}.\:;,@]|
        \p{Letter}
        "#,
    )
    .unwrap();

    let tokens: Vec<_> = re
        .find_iter(text)
        .map(|capture| capture.as_str().to_string())
        .collect();
    let a = KnotFinder(parse_expr(&mut tokens.into_iter().peekable()).unwrap());
    println!("{:?}", a);
    a
}

fn parse_union(tokens: &mut Peekable<std::vec::IntoIter<String>>) -> Option<SearchType> {
    if consume_if_equals(tokens, "(") {
        let result = parse_union(tokens);
        assert_eq!(tokens.next(), Some(")".to_string()));
        return result;
    }

    let mut ls = vec![];
    loop {
        let name = tokens.peek();
        let function = match name {
            None => break,
            Some(other) => match other.as_ref() {
                "stab" | "stabilize" | "st" => {
                    knot_stab
                }
                "commute" | "c" => {
                    knot_commute
                }
                "switch" | "sw" => {
                    knot_switch
                },
                "epsilon" | "e" => {
                    knot_epsilon
                },
                _ => break,
            },
        };
        tokens.next();
        ls.push(function);
    }

    Some(SearchType::Union(UnionSearchType {
        contains: Box::new(ls),
    }))
}

fn parse_expr(tokens: &mut Peekable<std::vec::IntoIter<String>>) -> Option<SearchType> {
    if consume_if_equals(tokens, "(") {
        let result = parse_expr(tokens);
        assert_eq!(tokens.next(), Some(")".to_string()));
        return result;
    }

    if consume_if_equals(tokens, "[") {
        let result = parse_union(tokens);
        assert_eq!(tokens.next(), Some("]".to_string()));
        return result;
    }

    if let Ok(count) = tokens.peek().unwrap().parse::<i32>() {
        tokens.next();
        return Some(SearchType::Repeat(RepeatSearchType {
            count,
            curr: 0,
            contains: Box::new(parse_expr(tokens).unwrap()),
        }));
    }

    let mut ls = vec![];
    loop {
        let name = tokens.peek();
        let function = match name {
            None => break,
            Some(other) => match other.as_ref() {
                "stab" | "stabilize" | "st" => {
                    tokens.next();
                    SearchType::Function(knot_stab)
                }
                "commute" | "c" => {
                    tokens.next();
                    SearchType::Function(knot_commute)
                }
                "switch" | "sw" => {
                    tokens.next();
                    SearchType::Function(knot_switch)
                }
                "(" | "[" => parse_expr(tokens).unwrap(),
                _ => break,
            },
        };
        ls.push(function);
    }

    Some(SearchType::List(ListSearchType {
        contains: Box::new(ls),
        curr: 0,
    }))
}

pub fn consume_if_equals(
    tokens: &mut Peekable<std::vec::IntoIter<String>>,
    expectation: &str,
) -> bool {
    let next_token = tokens.peek();
    if next_token.is_none_or(|a| a != expectation) {
        false
    } else {
        tokens.next();
        true
    }
}
