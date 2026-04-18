# deob

deob is a command-line tool that animates text by scrambling characters with configurable noise before revealing them, producing a decryption-style terminal effect.

## How it works

Each character in the input is shown as random noise for a configurable number of scramble cycles before snapping to its final value. Noise can be restricted to ASCII, alphanumeric, or hacker-style character sets; the default selects the set automatically based on the input. A `--markers` flag lets you delimit regions within a string so only those parts animate while the rest is shown statically.

## Prerequisites

Rust 1.70 or later (install via [rustup](https://rustup.rs)).

## Installation

```bash
cargo install --path .
```

Or build and copy manually:

```bash
make build
sudo make install
```

## Usage

Animate a single string:

```bash
deob "Hello, World!"
```

Read from stdin:

```bash
echo "Hello, World!" | deob
```

Animate side-by-side columns from files:

```bash
deob --col examples/entries.txt --col examples/entries.txt
```

Control the animation:

```bash
deob --speed 30 --color cyan --charset ascii --order random "Hello"
```

Use marker regions so only delimited text scrambles:

```bash
deob --markers "OS: ~Ubuntu~ kernel: ~6.1~"
```

## Example

Running `deob "deob"` produces a brief scramble before the word snaps into place. With `--color match` the noise inherits the ANSI color of each character.

## Testing

```bash
make test
```

To verify formatting before committing:

```bash
cargo fmt
make fmt
```

To run the linter:

```bash
make lint
```

## License

MIT, see [LICENSE](LICENSE).
