# ledgible

Formatter for [`ledger`](https://ledger-cli.org/) and
[`hledger`](https://hledger.org/) journals. This is **not** ready for production
use.

Rust version 1.89.0 or later is required to compile.

## Installation

After installing [`rustup`](https://rustup.rs),

`cargo install ledgible`

or, for the development version,

`cargo install ledgible --git https://git.sr.ht/~eleanor/ledgible --branch main`

## Options

```txt
Formatter for hledger journals

Usage: ledgible [OPTIONS] [FILE]

Arguments:
  [FILE]  Input journal

Options:
  -o, --output <FILE>  Write formatted journal to file
  -i, --inplace        Overwrite input file
  -e, --env            Read journal from $LEDGER_FILE
  -h, --help           Print help
  -V, --version        Print version
```

## Contributing

All contributions shall be licensed under GPLv3 or later. The `./commit.sh`
script in the repository should be run in order to ensure the commit meets
expectations.

### `./commit.sh` Requirements

- [`rustup`](https://rustup.rs)
- [`cargo-msrv`](https://github.com/foresterre/cargo-msrv)
- [`rust-clippy`](https://github.com/rust-lang/rust-clippy)
- [`rustfmt`](https://github.com/rust-lang/rustfmt)
- [`markdownlint-cli`](https://github.com/igorshubovych/markdownlint-cli)

Installation steps will vary from platform to platform. Assuming `rustup` is
installed, the other requirements can be installed with something like the
following:

```sh
cargo install cargo-msrv
rustup component add clippy
rustup component add rustfmt
npm install -g markdownlint-cli
```

Both `rustup` and `markdownlint-cli` are available via package managers such as
[Homebrew](https://brew.sh) or
[pacman](https://wiki.archlinux.org/title/Pacman).

## Roadmap

In no particular order:

- [ ] Add tests
- [ ] Remove extra spaces between amount and currency
- [ ] Sort postings to be consistent across transactions
- [ ] Align posting amounts to decimal place
- [ ] Handle negative sign on other side of currency
- [ ] Parse numbers with spaces in them (e.g. $10 000 or $- 100)
- [ ] Remove empty lines at top of journal
- [ ] Fully conform to ledger journal standard
- [ ] Standardize dates
- [ ] Per-currency/commodity formatting
- [ ] Expand tabs
- [ ] Parse split amounts for improved formatting
- [ ] Use user preference for `,` versus `.` in numbers
- [x] Remove trailing empty lines
- [x] Write output to file
- [x] Read from stdin by default
- [x] Standardize split amounts
