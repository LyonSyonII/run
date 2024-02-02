[![Logo](assets/logo.svg)](https://crates.io/crates/runfile)


[![Crates.io Version](https://img.shields.io/crates/v/runfile?style=flat)](https://crates.io/crates/runfile)

> Run commands in the languages you love!

![Screenshot](assets/screenshot.png)

Create scripts with multiple commands easily with `run`.  
You can write a command in any of the [supported languages](#languages), and even mix them!

And, if you have [`nix`](https://nixos.org/) installed, `run` will fetch the necessary packages automatically, there's no need to have any toolchain installed.

## Features
- Run commands in multiple languages seamlessly
- Fetch language toolchains automatically with `nix`
- Module system to reuse scripts
- Supports both commands and subcommands
- Self-contained in a single binary, no extra dependencies
- `runfiles` are just text files, easy to share and version control

## Languages
`run` supports the following languages, with their respective command indicators:

- [Rust](https://www.rust-lang.org/)
  - `rust | rs`
- [Python](https://www.python.org/)
  - `python | py`
- [Javascript](https://nodejs.org/)
  - `javascript | js`
- [Shell](https://en.wikipedia.org/wiki/Shell_script)
  - `shell | sh`
- [Bash](https://en.wikipedia.org/wiki/Bash_(Unix_shell))
  - `bash`
- [C#](https://docs.microsoft.com/en-us/dotnet/csharp/)
  - `csharp | cs | c#`
- [C](https://en.wikipedia.org/wiki/C_(programming_language))
  - `c`
- [C++](https://en.wikipedia.org/wiki/C%2B%2B)
  - `cplusplus | cpp | c++`

If you want a language to be added, please open an issue or a pull request!

## Installation
You can install `run` with `cargo`:

```sh
cargo install runfile
```

Or download the latest installer from the [releases page]()