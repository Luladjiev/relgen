# Relgen

`relgen` is an opinionated command-line interface (CLI) tool, designed to simplify the way of creating release Pull
Requests used in git-ops workflows.

## Usage

```bash
relgen --owner luladjiev --repo relgen --head main --base prod-branch --base-name Production --reviewer Luladjiev
```

`--repo` and `--reviewer` can be used multiple times to create Pull Requests in multiple repositories and assign reviews
to multiple persons

Relgen will not create a Pull Request if `head` branch is not ahead of `base` branch.

To get a comprehensive list of all available commands and options, you can use the `--help` flag:

```bash
relgen --help
```

## Installation

### Using Cargo

Installing `relgen` through Cargo is the easiest way to get started. You can install it by running the following
command:

```bash
cargo install relgen
```

### Building from source

You can also build `relgen` from source by running the following command:

```bash
cargo install --path .
```

## Development

`relgen` is developed using the [Rust programming language](https://www.rust-lang.org/) and
the [Cargo package manager](https://doc.rust-lang.org/cargo/).

You can clone the repository
and run the project locally using the following commands:

```bash
git clone https://github.com/luladjiev/relgen.git
cd relgen
cargo run
```

## Contributing

We welcome contributions from the community! Feel free to submit a Pull Request or open an issue if you find any bugs or
have suggestions for improvements.

## License

`relgen` is licensed under the [MIT License](https://choosealicense.com/licenses/mit/), a permissive license that lets
you do anything with the code with proper attribution and without warranty.
