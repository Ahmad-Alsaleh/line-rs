# Project Roadmap

## Phase 1: Errors

- [x] if line is out of bound, print: `error: line X not found`.
- [x] if the file is binary `error: non-text file`, unless a flag is passed, like in `bat`.
- [x] make sure -n=0 throws an error.

## Phase 2: Line Selection

- [x] support negative indexing (like in Python).
- [ ] (update: maybe no need) add a flag to preserve original order.
- [x] add syntax for range (eg: -n 1:4 and 1: and :4 and -2: and :-2) just like python.
- [x] add syntax for multiple lines (eg: -n 1,4).
- [x] support range + multiple lines (eg: -n 1:4,6). one way to implement that is to split on `,` then check if each part has `:` or not and store that in an `enum {Single(isize), Range(std::Range)}` or use a recursive `enum LineSelector {Single(isize), Range(isize, isize or std::Range), Multiple(Vec<LineSelector>)}`.
- [x] support steps in range, like in python (eg: -n 1:9:2 or ::-1).

## Phase 3: Output Enhancements 

- [x] add a flag `--context` to print the lines before and after the specified line, default value is zero.
- [x] do the same as `--context` but `--before` and `--after` (make sure `-a` and `-b` can't be used with `-c`, but `-a` and `-b` can be used together).
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
- [ ] add an `--array` flag to output in the format: `["two", "four"]`. Maybe no need for serde here, just use Vec Debug.
- [ ] add a `--json-line` flag to output in the format:
```
{"number":2,"content":"foo"}
{"number":4,"content":"bar"}
```

## Phase 5: Simple Enhancements

- [x] (update: no need) ~add a flag to print to stderr instead of stdout.~
- [ ] add a flag to use zero index.
- [ ] (important) read from stdin if no file was passed or when `-` is passed.
- [x] (update: no need, we don't have warnings anymore) ~make a `--quiet` flag to suppress warnings.~
- [ ] create custom error enum.

## Phase 6: Pretty Printing and Colors

- [x] (update: done, although olive is not used to reduce binary size) use pretty print (consider using olive!), something similar to `bat` style (print line number, file name, a border, etc), but add a flag to make it plain, and make it plain by default if redirection is detected (check how bat does that).
- [x] add colored line highlighting with a `--color` flag. this has nothing to do with pretty printing like (line numbers, border, file name, etc).
- [ ] respect the [NO_COLOR](https://no-color.org) env var.
- [x] (update: no need, we don't have colors anymore) ~color the word `Warning` in yellow.~

## Phase 7: Documentation and Packaging

- [ ] create GHA workflows to run tests when pushing to main. 
- [ ] updated the readme.
- [ ] consider making a man page.
- [ ] make this tool installable through `apt`, `cargo install`, and `brew` (what else?).
- [ ] add shell completions for bash, zsh, fish (what else?).
- [ ] run benchmarks to test if this tool is more efficient than other tools (eg: awk, sed, head + tail).
- [ ] benchmark memory usage for large files.

## Phase 8: Extra Features

- [ ] (maybe no need, overkill) add a flag `--not` to skip lines. the syntax should be the same as `-n` (range, multiple lines, negative values, etc.). make sure to `AND` the `-not` with `-n`. if `-n` is not there then print all lines except `--not`.
- [ ] (maybe no need, overkill) add option `--skip` to print `-n` except the skipped lines. the syntax for `--skip` is the same as `-n` (range, multiple lines, negative values, etc.).
- [ ] allow duplicates by default, set a flag to turn this off called `--no-duplicate`. This flag will turn off headers (but not line numbers and colors).
- [ ] add property-based tests.

## Phase 9: Performance and Security Enhancements
- [ ] try to optimize this tool when stdout is a pipe, e.g.: in `line -n=1:10000 file.txt | head -n 2`, line-rs shouldn't generate all 10000 line. this can be done by printing as soon as lines are read (see threading point below).
- [ ] consider using mmap for large files.
- [ ] consider using splice for large files.
- [ ] add path traversal protection.
- [ ] try multithreading: one thread will find the positions of all '\n' and the other thread will parse the line selectors and store the selected lines into the hashmap. maybe do this for large files only, since the overhead of multithreading will not be worth it (i am not very sure how this will improve performance).
- [ ] alternatively, use a channel and two threads. one thread will read lines and send the line to the channel, and the other thread will read from the channel and print the output.
