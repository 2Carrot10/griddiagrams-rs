use std::{iter::Peekable, rc::Rc};

use crate::{
    knot_core::DirList,
    reidemiester::{knot_commute, knot_stab, knot_switch},
};
use regex::Regex;
#[derive(Clone, Debug)]
struct ListSearchType {
    pub contains: Box<Vec<SearchType>>,
    curr: usize,
}

impl ListSearchType {
    pub fn next(&mut self) -> Option<fn(&DirList) -> Vec<DirList>> {
        if self.curr < self.contains.len() {
            let mut result = self.contains[self.curr].clone();
            self.curr += 1;
            result.next()
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
struct RepeatSearchType {
    pub count: i32,
    pub contains: Box<SearchType>,
    curr: i32,
}

impl RepeatSearchType {
    pub fn next(&mut self) -> Option<fn(&DirList) -> Vec<DirList>> {
        if self.curr < self.count {
            self.curr += 1;
            self.contains.next()
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
}

impl SearchType {
    pub fn next(&mut self) -> Option<fn(&DirList) -> Vec<DirList>> {
        match self {
            SearchType::Function(f) => Some(*f),
            SearchType::Repeat(f) => f.next(),
            SearchType::List(f) => f.next(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct KnotFinder(SearchType);

impl KnotFinder {
    pub fn next(&mut self) -> Option<fn(&DirList) -> Vec<DirList>> {
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

// Matches something like (destab) 10
fn parse_expr(tokens: &mut Peekable<std::vec::IntoIter<String>>) -> Option<SearchType> {
    println!("New");
    let mut ls = vec![];
    loop {
        let name = tokens.peek();
        println!("Name: {:?}", name);
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
                "(" => {
                    tokens.next();
                    let parse = parse_expr(tokens).unwrap();
                    println!("And now {:?}", tokens.peek());
                    assert_eq!(tokens.next(), Some(")".to_string()));

                    println!("Num now {:?}", tokens.peek());
                    let count: i32 = tokens.next().unwrap().parse().unwrap();
                    SearchType::Repeat(RepeatSearchType {
                        count: count,
                        contains: Box::new(parse),
                        curr: 0,
                    })
                }
                _ => break,
            },
        };
        ls.push(function);
    }
    println!("Break with {:?}", tokens.peek());

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
