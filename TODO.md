# TODO

- [ ] read from stdin if no file was passed (and maybe support `-` as file).
- [ ] add a `--json` flag
- [ ] make a `--pretty-json` flag. this will be helpful for example in: `result=$(line -n=2,4 --json file.txt | jq '.lines[0].content')`
```
{
  "source": "file.txt", // or "stdin"
  "lines": [
    { "number": 2, "content": "two" },
    { "number": 4, "content": "four" }
  ]
}
```
- [ ] add a `--sep` flag to specify the separator eg: `--sep=,` or `--sep=','` will print: `one,two,three` in plain text (without pretty printing)
- [ ] add a `--json-array` flag to output in the format: `["two", "four"]`
- [ ] add a `--json-line` flag to output in the format:
```
{"number":2,"content":"foo"}
{"number":4,"content":"bar"}
```
- [ ] support negative indexing (like in Python).
- [ ] think of how to handle out-of-bound cases, error by default? or warn? add a flag `--force` or `--silent` to override the default the behaviour.
- [ ] add a flag to print to stderr instead of stdout.
- [ ] use pretty print (consider using olive!), something similar to `bat` style (print line number, file name, a border, etc), but add a flag to make it plain, and make it plain by default if redirection is detected (check how bat does that).
- [ ] add a flag to zero index.
- [ ] write test cases.
- [ ] add syntax for range (eg: -n 1:4 and 1: and :4 and -2: and :-2) just like python.
- [ ] add syntax for multiple lines (eg: -n 1,4).
- [ ] support range + multiple lines (eg: -n 1:4,6). one way to implement that is to split on `,` then check if each part has `:` or not and store that in an `enum {Single(isize), Range(std::Range)}` or use a recursive `enum LineSelector {Single(isize), Range(isize, isize or std::Range), Multiple(Vec<LineSelector>)}`
- [ ] support steps in range, like in python (eg: -n 1:9:2 or ::-1)
- [ ] add a flag `--not` to skip lines. the syntax should be the same as `-n` (range, multiple lines, negative values, etc.). make sure to `AND` the `-not` with `-n`. if `-n` is not there then print all lines except `--not`.
- [ ] add option `--skip` to print `-n` except the skipped lines. the syntax for `--skip` is the same as `-n` (range, multiple lines, negative values, etc.).
- [ ] deduplicate lines by default, set a flag to turn this off called `--duplicate`.
- [ ] add colored line highlighting with a `--color` flag.
- [ ] make this tool installable through `apt`, `cargo install`, and `brew` (what else?).
- [ ] add shell completions for bash, zsh, fish.
- [ ] make sure -n=0 throws an error.
- [ ] if line is out of bound, print: `error: line X not found`.
- [ ] if the file is binary `error: non-text file`.
- [ ] add a flag `--context` to print the lines before and after the specified line.
- [ ] consider making a man page
- [ ] add colored line highlighting with a `--color` flag. this has nothing to do with pretty printing like `bat`
- [ ] add a readme. check the following by gpt. make sure it matches other famous rust cli tools and make sure it doesn't sound like ai generated

# README:

# line-rs

`line-rs` is a simple command-line tool to extract specific lines from a text file — without relying on shell pipelines like `head` and `tail`.

---

## 🚀 Features

- Extract a single line:  
  ```sh
  line -n=5 file.txt
  ```

* Extract a range of lines:

  ```sh
  line -n=2:4 file.txt
  ```

* Extract multiple specific lines:

  ```sh
  line -n=2,4,6 file.txt
  ```

* Input from stdin:

  ```sh
  cat file.txt | line -n=3 -
  ```

---

## 🔧 Installation

If you have Rust installed:

```sh
cargo install line-rs
```

---

## 📝 Usage

```
Usage: line -n=<lines> <file>

Options:
  -n=<lines>   Lines to extract (e.g., 3, 2:5, 4,6,10)
  -            Use "-" as input file to read from stdin
```

---

## 📦 Examples

```sh
# Print line 5
line -n=5 notes.txt

# Print lines 3 to 7
line -n=3:7 notes.txt

# Print lines 2, 4, and 6
line -n=2,4,6 notes.txt

# Print line 1 from stdin
echo -e "a\nb\nc" | line -n=1 -
```

---

## 🛠 Motivation

This tool simplifies extracting lines from a file or stream without relying on brittle chains like:

```sh
cat file.txt | head -n=4 | tail -n=1
```

`line-rs` gives you a cleaner and more flexible alternative.

---

## 📃 License

MIT

---

## 💬 Feedback

Feel free to open issues or suggestions on [GitHub](https://github.com/yourname/line-rs).

```

Let me know if you'd like me to add sections for contributing, testing, or packaging (e.g., Debian, Homebrew, etc).
```

