# TODO

## Phase 1: Errors

- [x] if line is out of bound, print: `error: line X not found`.
- [x] if the file is binary `error: non-text file`, unless a flag is passed, like in `bat`.
- [x] make sure -n=0 throws an error.

## Phase 2: Line Selection

- [x] support negative indexing (like in Python).
- [ ] add a flag to preserve original order.
- [ ] add syntax for range (eg: -n 1:4 and 1: and :4 and -2: and :-2) just like python.
- [ ] add syntax for multiple lines (eg: -n 1,4).
- [ ] support range + multiple lines (eg: -n 1:4,6). one way to implement that is to split on `,` then check if each part has `:` or not and store that in an `enum {Single(isize), Range(std::Range)}` or use a recursive `enum LineSelector {Single(isize), Range(isize, isize or std::Range), Multiple(Vec<LineSelector>)}`.
- [ ] support steps in range, like in python (eg: -n 1:9:2 or ::-1).

## Phase 3: Output Enhancements 

- [ ] add a flag `--context` to print the lines before and after the specified line.
- [ ] add a `--sep` flag to specify the separator eg: `--sep=','` will print: `one,two,three` in plain text (without pretty printing). default is '\n'.

## Phase 4: Json Printing

- [ ] add a `--json` flag to print a dict in one line.
- [ ] make a `--pretty-json` flag. this will be helpful for example in: `result=$(line -n=2,4 --json file.txt | jq '.lines[0].content')`:
```
{
  "source": "file.txt", // or "stdin"
  "lines": [
    { "number": 2, "content": "two" },
    { "number": 4, "content": "four" }
  ]
}
```
- [ ] add a `---array` flag to output in the format: `["two", "four"]`. Maybe no need for serde here, just use Vec Debug.
- [ ] add a `--json-line` flag to output in the format:
```
{"number":2,"content":"foo"}
{"number":4,"content":"bar"}
```

## Phase 5: Simple Enhancements

- [ ] add a flag to print to stderr instead of stdout.
- [ ] add a flag to use zero index.
- [ ] read from stdin if no file was passed (and maybe support `-` as file).
- [ ] make a `--quiet` flag to suppress warnings.

## Phase 6: Pretty Printing and Colors

- [ ] use pretty print (consider using olive!), something similar to `bat` style (print line number, file name, a border, etc), but add a flag to make it plain, and make it plain by default if redirection is detected (check how bat does that).
- [ ] add colored line highlighting with a `--color` flag. this has nothing to do with pretty printing like (line numbers, border, file name, etc).
- [ ] respect the [NO_COLOR](https://no-color.org) env var.
- [ ] color the word `Warning` in yellow.

## Pahse 7: Documentation and Packaging

- [ ] add a readme. check the following by gpt. make sure it matches other famous rust cli tools and make sure it doesn't sound like ai generated.
- [ ] consider making a man page.
- [ ] make this tool installable through `apt`, `cargo install`, and `brew` (what else?).
- [ ] add shell completions for bash, zsh, fish.

## Phase 8: Extra Features

- [ ] add a flag `--not` to skip lines. the syntax should be the same as `-n` (range, multiple lines, negative values, etc.). make sure to `AND` the `-not` with `-n`. if `-n` is not there then print all lines except `--not`.
- [ ] add option `--skip` to print `-n` except the skipped lines. the syntax for `--skip` is the same as `-n` (range, multiple lines, negative values, etc.).
- [ ] deduplicate lines by default, set a flag to turn this off called `--duplicate`.

