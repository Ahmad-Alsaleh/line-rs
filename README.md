# line-rs

`line-rs` is a simple command-line tool to extract specific lines from a text file without hacks like `head -n 5 filename.txt | tail -n +2`.

![demo will be here soon](demo.png "`line-rs` Demo")

## Features

### Line Selection

`line-rs` supports three main ways to select lines:

1. Select a single line:

  ```sh
  line -n=5 file.txt
  ```

2. Select a range of lines (both ends are included):

  ```sh
  line -n=2:4 file.txt # 2, 3, 4
  ```

3. Select multiple specific lines:

  ```sh
  line -n=2,4,6 file.txt
  ```

You can also combine the above! The command bellow selects the line `2` and the range `4:6`.

  ```sh
  line -n=2,4:6 file.txt # 2, 4, 5, 6
  ```

### Advanced Line Selection

`line-rs` supports line selection in a powerful way. If you are familiar with Python syntax, the following should be familiar as well:

- Negative indexing:
    - Select the last line:
        ```sh
        line -n=-1
        ```

    - From line 3 up to the 2 line from the end:
        ```sh
        line -n=3:-2
        ```

- Unbounded Ranges:
    - From line 3 up to the end of the file:
        ```sh
        line -n=3:
        ```

    - From the beginning of the file up to line 9:
        ```sh
        line -n=:9 # equivalent to 1:9
        ```

    - All lines, equivalent to `cat`:
        ```sh
        line -n=:
        ```
- Steps:
    - From line 3 up to 9, jumping two lines at a time: 
        ```sh
        line -n=3:7:2 # 3, 5, 7
        ```
    - From line 9 up to 3, jumping backwards:
    ```sh
    line -n=5:3:-1 # 5, 4, 3
    ```
    - Reverse all lines:
    ```sh
    line -n=::-1
    ```

You can also skip lines easily:

- Print all line except 5 and 7:
    ```sh
    line --skip=5,7
    ```

- Print line 2 onwards, skipping lines 6, 7, and 8:
    ```sh
    line -n=2: --skip=6:8
    ```

### Pretty Printing

> More about that soon

### Json Output

The output can be serialized as JSON, useful for piping and scripts

```sh
line -n=2,4 --json # one line, useful for piping
line -n=2,4 --pretty-json
```

Output:
```sh
{
  "source": "file.txt", // or "stdin"
  "lines": [
    { "number": 2, "content": "hi" },
    { "number": 4, "content": "hello" }
  ]
}
```

```sh
line -n=2,4 --array
```

Output
```sh
["hi", "hello"]
```

```sh
line -n=2,4 --json-line
```

Output
```sh
{"number":2,"content":"foo"}
{"number":4,"content":"bar"}
```

## Installation

> More about that soon

### Using Cargo

### Using pip

### Using apt

### Using brew

## Shell Completion

> More about that soon

## Examples

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

## Motivation

This tool simplifies extracting lines from a file or stream without relying on brittle chains like:

```sh
cat file.txt | head -n=4 | tail -n=1
```

`line-rs` gives you a cleaner and more flexible alternative.

---

## License

MIT

---

## Feedback

Feel free to open issues or suggestions on [GitHub](https://github.com/Ahmad-Alsaleh/line-rs).

