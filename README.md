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
./griddiagrams -knots 12n_79 -algorithm algorithms/commute-or-stab --logging multi --verbose-output -o output/out.json 
```
To search all knots using just commutations, logging only weather or not they have succeeded:
```sh
./griddiagrams -knots all -algorithm commute --logging none -o output/out.json 
```
