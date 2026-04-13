use std::iter::Peekable;

use crate::{
    knot_core::DirList,
    reidemiester::{knot_commute, knot_destab, knot_epsilon, knot_stab, knot_switch},
};

use regex::Regex;

type MoveFunction = fn(&DirList) -> Vec<DirList>;
type DynamicMoveFunction = Box<dyn Fn(&DirList) -> Vec<DirList> + Sync>;

trait AlgorithmGrammer {
    fn next(&mut self) -> Option<(DynamicMoveFunction, String)>;
}

#[derive(Clone, Debug)]
pub struct ListSearchType {
    pub contains: Box<Vec<SearchType>>,
    curr: usize,
}

impl ListSearchType {
    pub fn new(contains: Vec<SearchType>) -> ListSearchType {
        ListSearchType {
            contains: Box::new(contains),
            curr: 0,
        }
    }
}
impl AlgorithmGrammer for ListSearchType {
    fn next(&mut self) -> Option<(DynamicMoveFunction, String)> {
        if self.curr < self.contains.len() {
            let result = &mut self.contains[self.curr];
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

#[derive(Clone, Debug)]
pub struct UnionSearchType {
    pub contains: Vec<(MoveFunction, String)>,
}

impl AlgorithmGrammer for UnionSearchType {
    fn next(&mut self) -> Option<(DynamicMoveFunction, String)> {
        if self.contains.is_empty() {
            return None;
        }
        let mut name: String = "".to_string();
        for (_, new_name) in &self.contains {
            name = format!("{} | {}", name, new_name);
        }

        let contains = self.contains.clone();

        Some((
            Box::new(move |input: &DirList| {
                let mut out = Vec::new();
                for (fun, _) in &contains {
                    out.extend(fun(input));
                }
                out
            }),
            name,
        ))
    }
}

#[derive(Clone, Debug)]
pub struct RepeatSearchType {
    pub count: i32,
    pub contains: Box<SearchType>,
    pub curr: i32,
}

impl AlgorithmGrammer for RepeatSearchType {
    fn next(&mut self) -> Option<(DynamicMoveFunction, String)> {
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
pub struct FunctionSearchType {
    function: (MoveFunction, String),
    contains: bool,
}
impl FunctionSearchType {
    pub fn new(func: MoveFunction, name: String) -> Self {
        FunctionSearchType {
            function: (func, name),
            contains: true,
        }
    }
}

impl AlgorithmGrammer for FunctionSearchType {
    fn next(&mut self) -> Option<(DynamicMoveFunction, String)> {
        self.contains = !self.contains;
        if self.contains {
            None
        } else {
            Some((Box::new(self.function.0.clone()), self.function.1.clone()))
        }
    }
}

#[derive(Clone, Debug)]
pub enum SearchType {
    Function(FunctionSearchType),
    Repeat(RepeatSearchType),
    List(ListSearchType),
    Union(UnionSearchType),
}

impl AlgorithmGrammer for SearchType {
    fn next(&mut self) -> Option<(DynamicMoveFunction, String)> {
        match self {
            SearchType::Function(f) => f.next(),
            SearchType::Repeat(f) => f.next(),
            SearchType::List(f) => f.next(),
            SearchType::Union(f) => f.next(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct KnotFinder(pub SearchType);

impl KnotFinder {
    pub fn next(&mut self) -> Option<(DynamicMoveFunction, String)> {
        self.0.next()
    }

    pub fn build_search_type(
        depth: i32,
        function: fn(&DirList) -> Vec<DirList>,
        name: String,
    ) -> KnotFinder {
        KnotFinder(SearchType::Repeat(RepeatSearchType {
            contains: Box::new(SearchType::Function(FunctionSearchType::new(
                function, name,
            ))),
            count: depth,
            curr: 0,
        }))
    }

    pub fn build(depth: i32, arg: SearchType) -> KnotFinder {
        KnotFinder(SearchType::Repeat(RepeatSearchType {
            contains: Box::new(arg),
            count: depth,
            curr: 0,
        }))
    }
}

pub fn stab_search(depth: i32) -> KnotFinder {
    KnotFinder(SearchType::List(ListSearchType::new(vec![
        SearchType::Function(FunctionSearchType::new(knot_stab, String::from("stab"))),
        SearchType::Repeat(RepeatSearchType {
            contains: Box::new(SearchType::Function(FunctionSearchType::new(
                knot_commute,
                String::from("commute"),
            ))),
            count: depth,
            curr: 0,
        }),
    ])))
}

pub fn commute_search(depth: i32) -> KnotFinder {
    KnotFinder::build_search_type(depth, knot_commute, String::from("commute"))
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

    let mut tokens = re
        .find_iter(text)
        .map(|capture| capture.as_str().to_string())
        .collect::<Vec<_>>().into_iter().peekable();
    let a = KnotFinder(parse_expr(&mut tokens).unwrap());

    assert_eq!(tokens.peek(), None, "Parser failed");
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
                "stab" | "stabilize" | "st" => (knot_stab as MoveFunction, String::from("stab")),
                "commute" | "c" => (knot_commute as MoveFunction, String::from("commute")),
                "switch" | "sw" => (knot_switch as MoveFunction, String::from("switch")),
                "destab" | "destabilize" | "d" => {
                    (knot_destab as MoveFunction, String::from("destab"))
                }
                "epsilon" | "e" => (knot_epsilon as MoveFunction, String::from("epsilon")),
                _ => break,
            },
        };
        tokens.next();
        ls.push(function);
    }

    Some(SearchType::Union(UnionSearchType { contains: ls }))
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
                    SearchType::Function(FunctionSearchType::new(knot_stab, String::from("stab")))
                }
                "commute" | "c" => {
                    tokens.next();
                    SearchType::Function(FunctionSearchType::new(knot_commute, String::from("commute")))
                }
                "switch" | "sw" => {
                    tokens.next();
                    SearchType::Function(FunctionSearchType::new(knot_switch, String::from("switch")))
                }
                "destab" | "d" => {
                    tokens.next();
                    SearchType::Function(FunctionSearchType::new(knot_destab, String::from("destab")))
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
