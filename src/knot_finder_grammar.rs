use std::iter::Peekable;

use crate::{
    knot_core::DirList,
    reidemiester::{knot_commute, knot_destab, knot_epsilon, knot_stab, knot_switch},
};

use regex::Regex;

type MoveFunction = fn(&DirList) -> Vec<(DirList, String)>;
type DynamicMoveFunction = Box<dyn Fn(&DirList) -> Vec<(DirList, String)> + Sync>;

type MoveConfiguration<'a> = (DynamicMoveFunction, &'a MoveOptions);

#[derive(Clone, Debug)]
struct MoveOptions {
    name: String,
    dedup: bool,
    verify_niceness: bool
}

trait AlgorithmGrammar {
    fn next<'a>(&'a mut self) -> Option<MoveConfiguration<'a>>;
}

#[derive(Clone, Debug)]
pub struct ListSearchType {
    pub contains: Vec<SearchType>,
    curr: usize,
}

impl ListSearchType {
    pub fn new(contains: Vec<SearchType>) -> ListSearchType {
        ListSearchType {
            contains,
            curr: 0,
        }
    }
}

impl AlgorithmGrammar for ListSearchType {
    fn next<'a>(&'a mut self) -> Option<MoveConfiguration<'a>> {
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
    current: bool,
}

impl AlgorithmGrammar for UnionSearchType {
    fn next(&mut self) -> Option<MoveConfiguration> {
        if !self.current {
            self.current = !self.current;
            return None;
        }

        if self.contains.is_empty() {
            return None;
        }
        let mut name: String = "".to_string();
        for (_, new_name) in &self.contains {
            if &name != "" {
                name = format!("{} | {}", name, new_name);
            } else {
                name = new_name.clone();
            }
        }

        let contains = self.contains.clone();
        self.current = !self.current;

        Some((
            Box::new(move |input: &DirList| {
                let mut out = Vec::new();
                for (fun, _) in &contains {
                    out.extend(fun(input));
                }
                out
            }),
            name,
            true,
        ))
    }
}

#[derive(Clone, Debug)]
pub struct RepeatSearchType {
    pub count: i32,
    pub contains: Box<SearchType>,
    pub curr: i32,
}

impl AlgorithmGrammar for RepeatSearchType {
    fn next<'a>(&'a mut self) -> Option<MoveConfiguration<'a>> {
        if self.curr < self.count {
            if let Some(some) = self.contains.next() {
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
pub struct FunctionSearchType {
    move_config: (MoveFunction, MoveOptions),
    contains: bool,
}

impl FunctionSearchType {
    pub fn new(func: MoveFunction, name: String) -> Self {
        FunctionSearchType {
            move_config: (func,  MoveOptions {
                name,
                dedup: true,
                verify_niceness: true

            }),
            contains: true,
        }
    }
}

impl AlgorithmGrammar for FunctionSearchType {
    fn next<'a>(&'a mut self) -> Option<MoveConfiguration<'a>> {
        self.contains = !self.contains;
        if self.contains {
            None
        } else {
            Some((
                Box::new(self.move_config.0.clone()),
                &self.move_config.1
            ))
        }
    }
}

#[derive(Clone, Debug)]
pub struct IgnoreNiceness(Box<SearchType>);

impl AlgorithmGrammar for IgnoreNiceness {
    fn next(&mut self) -> Option<MoveConfiguration> {
        let x = self.0.next();
        x.;
        return x;
    }
}

#[derive(Clone, Debug)]
pub enum SearchType {
    Function(FunctionSearchType),
    Repeat(RepeatSearchType),
    List(ListSearchType),
    Union(UnionSearchType),
    IgnoreNiceness(IgnoreNiceness)
}

impl AlgorithmGrammar for SearchType {
    fn next(&mut self) -> Option<MoveConfiguration> {
        match self {
            SearchType::Function(f) => f.next(),
            SearchType::Repeat(f) => f.next(),
            SearchType::List(f) => f.next(),
            SearchType::Union(f) => f.next(),
            SearchType::IgnoreNiceness(f) => f.next(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct KnotFinder(pub SearchType);

impl KnotFinder {
    pub fn next(&mut self) -> Option<(DynamicMoveFunction, String, bool)> {
        self.0.next()
    }

    pub fn build_search_type(
        depth: i32,
        function: fn(&DirList) -> Vec<(DirList, String)>,
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

    #[allow(dead_code)]
    pub fn build(depth: i32, arg: SearchType) -> KnotFinder {
        KnotFinder(SearchType::Repeat(RepeatSearchType {
            contains: Box::new(arg),
            count: depth,
            curr: 0,
        }))
    }
}

pub fn stab_commute_search(depth: i32) -> KnotFinder {
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

    // tokenize
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
        .collect::<Vec<_>>()
        .into_iter()
        .peekable();
    let knot_finder = KnotFinder(parse_expr(&mut tokens).unwrap());

    assert_eq!(tokens.peek(), None, "Parser failed");
    knot_finder
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

    Some(SearchType::Union(UnionSearchType {
        contains: ls,
        current: true,
    }))
}

fn parse_parens(tokens: &mut Peekable<std::vec::IntoIter<String>>) -> Option<SearchType> {
    if consume_if_equals(tokens, "[") {
        let result = parse_expr(tokens);
        assert_eq!(tokens.next(), Some("]".to_string()));
        return result;
    }

    if consume_if_equals(tokens, "(") {
        let result = parse_expr(tokens);
        assert_eq!(tokens.next(), Some(")".to_string()));
        return result;
    }

    if consume_if_equals(tokens, ":") {
        let result = parse_expr(tokens);
        assert_eq!(tokens.next(), Some(":".to_string()));
        return result.map(|a| SearchType::IgnoreNiceness(IgnoreNiceness { contains: Box::new(a)}));
    }

    // deduplicate
    if consume_if_equals(tokens, "<") {
        let result = parse_expr(tokens);
        assert_eq!(tokens.next(), Some(">".to_string()));
        return result.map(|a| SearchType::VisitedDedup(VisitedDedup(Box::new(a))));
    }

    if consume_if_equals(tokens, "{") {
        let result = parse_union(tokens);
        assert_eq!(tokens.next(), Some("}".to_string()));
        return result;
    }
    panic!("Expected '{{', '[', or '<'. (this is likely an issue with the Rust code)");
}

fn parse_expr(tokens: &mut Peekable<std::vec::IntoIter<String>>) -> Option<SearchType> {
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
                    SearchType::Function(FunctionSearchType::new(
                        knot_commute,
                        String::from("commute"),
                    ))
                }
                "switch" | "sw" => {
                    tokens.next();
                    SearchType::Function(FunctionSearchType::new(
                        knot_switch,
                        String::from("switch"),
                    ))
                }
                "destab" | "d" => {
                    tokens.next();
                    SearchType::Function(FunctionSearchType::new(
                        knot_destab,
                        String::from("destab"),
                    ))
                }
                "{" | "[" | "(" | "<" => {
                    let a = parse_parens(tokens).unwrap();
                    a
                }
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
