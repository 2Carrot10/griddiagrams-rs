use std::{iter::Peekable, rc::Rc};

use crate::{
    knot_core::DirList,
    reidemiester::{knot_commute, knot_stab, knot_switch},
};
use regex::Regex;
#[derive(Clone)]
struct RepeatSearchType {
    pub count: i32,
    pub next: Rc<Vec<SearchType>>,
    curr: usize,
}

impl RepeatSearchType {
    pub fn next(&mut self) -> Option<fn(&DirList) -> Vec<DirList>> {
        if self.count == 0 {
            return None;
        } else if self.curr < self.next.len() {
            self.count += 1;
        } else {
            self.count -= 1;
            self.curr = 0;
        }
        match self.next[self.curr].clone() {
            SearchType::Function(f) => Some(f.clone()),
            SearchType::RepeatSearchType(mut r) => r.next(),
        }
    }
}

#[derive(Clone)]
enum SearchType {
    Function(fn(&DirList) -> Vec<DirList>),
    RepeatSearchType(RepeatSearchType),
}

#[derive(Clone)]
pub struct KnotFinder(RepeatSearchType);

impl KnotFinder {
    pub fn next(&mut self) -> Option<fn(&DirList) -> Vec<DirList>> {
        self.0.next()
    }

    pub fn build(depth: i32, function: fn(&DirList) -> Vec<DirList>) -> KnotFinder {
        KnotFinder(RepeatSearchType {
            next: Rc::new(vec![SearchType::Function(function)]),
            count: depth,
            curr: 0,
        })
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
    KnotFinder(parse_expr(&mut tokens.into_iter().peekable()).unwrap())
}

// Matches something like (destab 10)
fn parse_expr(tokens: &mut Peekable<std::vec::IntoIter<String>>) -> Option<RepeatSearchType> {
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
                    let t = parse_expr(tokens).unwrap();
                    println!("Okie {:?}", tokens.peek());

                    let count: i32 = tokens.next().unwrap().parse().unwrap();
                    let result = Some(RepeatSearchType {
                        count,
                        next: Rc::new(vec![SearchType::RepeatSearchType(t)]),
                        curr: 0,
                    });
                    assert_eq!(tokens.next(), Some(")".to_string()));
                    return result;
                    
                }
                _ => break,
            },
        };
        ls.push(function);
    }

    Some(RepeatSearchType {
        count: 1,
        next: Rc::new(ls),
        curr: 0,
    })
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
