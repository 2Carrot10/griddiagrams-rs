use regex::Regex;

use crate::knot_core::DirList;

/// Parses a string representation of a vertlist into the DirList struct
/// `text` - of the form `[(n, n), (n, n), (n, n) ...]` where n represents a column index, (n, n)
/// represents the indices of x and o at the row index given by the position in the array.  
pub fn string_to_vertmap(text: String) -> DirList {
    let mut out: DirList = DirList(vec![]);

    // A general purpose lexical analysis regular expression. 
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
        .find_iter(&text)
        .map(|capture| capture.as_str().to_string())
        .collect::<Vec<_>>()
        .into_iter()
        .peekable();

    assert_eq!(tokens.next().as_deref(), Some("["));
    while tokens.peek().map(|s| s.as_str()) != Some("]") {
        assert_eq!(tokens.next().as_deref(), Some("("));
        let x = tokens.next().unwrap().parse::<i32>().unwrap();
        assert_eq!(tokens.next().as_deref(), Some(","));
        let o = tokens.next().unwrap().parse::<i32>().unwrap();
        assert_eq!(tokens.next().as_deref(), Some(")"));
        if tokens.peek().map(|s| s.as_str()) != Some("]") {
            assert_eq!(tokens.next().as_deref(), Some(","));
        }
        out.0.push((x, o));
    }
    assert!(
        crate::knot_core::is_valid(&out),
        "Diagram is not a valid knot"
    );
    out
}
