# griddiagrams-rs
`griddiagrams-rs` is an improved, rust-based implementation of [griddiagrams](https://github.com/paulitzlinger/griddiagrams) and its [accompanying paper](https://arxiv.org/abs/2602.02642).
This tool is used to find "nice", griddiagrams of fibered knots. It has been used to build on the findings of `griddiagrams` by finding nice diagrams 12 previously unsolved knots: 12n79, 12n168, 13n282, 13n917, 13n1279, 13n1281, 13n1413, 13n1826, 13n2915, 13n3089, 13n3904, 13n3932

Main improvements:
- Increased speed
- Mutlithreading
- Improved code quality
- Custom search algorithm
- Easy-to-use CLI

# Setup
1. Install rustup
https://rust-lang.org/tools/install/
2. Install the nightly toolchain
```sh
rustup toolchain install nightly
```
3. Build the project in release mode
```sh
cargo +nightly build --release
```
Run the project 
```
./griddiagrams [OPTIONS]
```
# Usage
```
Options:
  -o, --output <file>              store record of search successes and failures in <file>
  -i, --input <file>               search knots using failures in <file>; can only be used in combination with `--knots rest`
  -k, --knots <KNOTS>              which knots to target ("all", comma separated knot names "<start index> - <end index>", "rest" (in combination with --input), or a vertlist in the format [(1,0), (0,1)] [default: all]
  -a, --algorithm <ALGORITHM>      which algorithm to use ("stabilize", "commute", or a path to an algorithm file) [default: stab]
      --logging <LOGGING>          how to format lines of logging when searching for a solution for a knot (none, single, multi) [default: single]
      --result-type <RESULT_TYPE>  which results to display (positives, negatives, both, neither) [default: both]
      --hide-analytics
  -n, --depth <DEPTH>              [default: 200]
  -t, --threads <THREADS>          Will default to the number of cores available on the system
      --hide-diagrams
      --verbose-output             store nice diagram instead of just weather or not it exists
  -h, --help                       Print help
  -V, --version                    Print version
```
# Search Algorithm Configuration
The search algorithm can be used to create complex sequences of reidemeister moves with which to search. Search algorithms are weaker in power to a regular language. Similarly to RegEx, elements placed in sequence are matched on in order.
By default, to create the subsequent frontier, the algorithm will only compute moves on the frontier. The following frontier cannot include previously visited nodes. This means that the calculated paths will include only in-between griddiagrams that can not be found more quickly using a different path. Deduplication based on the visited set improves performance, because most griddiagrams will not need to be searched. However, if you would like to disable this, use `< ... >`.
`{ ... }` is used to represent a union. Unlike regular languages, unions can not contain non-primitive elements. This limitation allows minor code improvements, as states do not need to be tagged.
```
<reidemeister moves> ::= "stabilize" | "commute" | "epsilon" | "switch"
<expr> ::= "(" <num repititions> <sequence> ")" | "<" <expr> <sequence> ">" | "{" <num repititions> <sequence> "}"
<sequence> ::= <reidemeister move> | <reidemeister move> "," <sequence>
<no-deduplicate> ::= "<" <num repititions> <reidemeister moves> ">"
```

For instance, this algorithm will do the following four times: 
1. stabilize
2. switch or commute without deduplicating based on the visited state 
3. destabilize
4. commute or switch 10 times

```
4 (
	stab
	< 3 { switch commute } >
	destab
	( 10 {commute switch })
)
```
# Example
To search 12n_79 (the first previously unsolved knot) using the algorithm found at `algorithms/commute-or-stab`, outputing to `output/out.json` with a large amount of logging:
```sh
./griddiagrams --knots 12n_79 --algorithm algorithms/commute-or-stab --logging multi --verbose-output -o output/out.json 
```
This command results in the following (abridged) output:
```
----
# 0: 12n_79
····╭──────╮
··╭─│─────╮│
··│·│····╭│╯
··│·│···╭│╯·
··│·│··╭│╯··
··│·│·╭│╯···
╭─│─│╮││····
│·│╭│││╯····
│╭││││╯·····
╰│││╯│······
·│╰│─╯······
·╰─╯········

1      Size of the frontier: 96          [▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒]  Ratio change: 9600.00%    stab de-duplicate
2      Size of the frontier: 410         [▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒]  Ratio change: 427.08%    commute de-duplicate
3      Size of the frontier: 1283        [▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒]  Ratio change: 312.93%    commute de-duplicate
4      Size of the frontier: 3095        [▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒------]  Ratio change: 241.23%    commute de-duplicate
5      Size of the frontier: 6680        [▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒---------]  Ratio change: 215.83%    commute de-duplicate
6      Size of the frontier: 12802       [▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒-----------]  Ratio change: 191.65%    commute de-duplicate
<iterations abridged for brevity>
18     Size of the frontier: 2479379     [▒▒▒▒▒▒▒▒▒▒▒▒▒▒----------------]  Ratio change: 143.62%    commute de-duplicate
19     Size of the frontier: 3522691     [▒▒▒▒▒▒▒▒▒▒▒▒▒▒----------------]  Ratio change: 142.08%    commute de-duplicate
20     Size of the frontier: 4953972     [▒▒▒▒▒▒▒▒▒▒▒▒▒▒----------------]  Ratio change: 140.63%    commute de-duplicate
Found nice knot for 12n_79, #0: [(0, 2), (9, 4), (1, 8), (6, 12), (10, 0), (11, 9), (8, 5), (4, 10), (7, 3), (2, 6), (5, 7), (3, 11), (12, 1)]
╭───╮········
│·╭─│───────╮
╰─│─│────╮··│
··│·│···╭│─╮│
·╭│─│──╮││·││
·││·│·╭│││╮││
·││╭│─│││╯│││
·││││·││╰─╯││
·│╰││─╯│···││
·╰─││╮·│···││
···│╰│─╯···││
···│·╰─────╯│
···╰────────╯
```
To search all knots using just commutations, logging only weather or not they have succeeded:
```sh
./griddiagrams --knots all --algorithm commute --logging none -o output/out.json 
```
This command results in the following (abridged) output
```
Found nice knot for 3_1, #0: [(2, 0), (3, 1), (4, 2), (0, 3), (1, 4)]
Found nice knot for 4_1, #1: [(2, 0), (1, 3), (5, 2), (0, 4), (3, 5), (4, 1)]
Found nice knot for 5_1, #2: [(2, 0), (3, 1), (4, 2), (5, 3), (6, 4), (0, 5), (1, 6)]
Found nice knot for 8_19, #3: [(3, 0), (4, 1), (5, 2), (6, 3), (0, 4), (1, 5), (2, 6)]
Found nice knot for 6_2, #4: [(2, 0), (3, 1), (4, 2), (6, 3), (5, 7), (0, 6), (7, 4), (1, 5)]
Found nice knot for 6_3, #5: [(2, 0), (3, 1), (5, 2), (4, 7), (0, 6), (7, 3), (1, 5), (6, 4)]
Found nice knot for 8_20, #6: [(3, 0), (4, 1), (2, 7), (5, 3), (0, 4), (1, 6), (7, 5), (6, 2)]
Could not find nice knot for 8_21, #7 (search space error)
Could not find nice knot for 9_42, #8 (search space error)
<iterations abridged for brevity>
Found nice knot for 13a_4844, #5394: [(11, 7), (9, 13), (6, 10), (8, 1), (12, 5), (7, 9), (10, 14), (4, 6), (13, 0), (5, 11), (14, 8), (3, 12), (2, 4), (1, 3), (0, 2)]
Could not find nice knot for 13a_4847, #5395 (search space error)
Found nice knot for 13a_4878, #5396: [(2, 0), (3, 1), (4, 2), (5, 3), (6, 4), (7, 5), (8, 6), (9, 7), (10, 8), (11, 9), (12, 10), (13, 11), (14, 12), (0, 13), (1, 14)]
========= Analytics =========
Total: 5397
Positive results: 4346   80%
Negative results: 1051   19%
   Depth error: 15   0%
   Frontier error: 1036   19%
```
