# TODO

## Features
- [] Rework function brackets
  - > End bracket must have same indentation as `LANG` or `CMD` (in case lange is not specified) or be in the same line
- [] Global variables
- [] Optional arguments
- [] --run <LANG\> option, runs stdin as that language
- [] Run commands from languages 
  - [] Shell/Bash (use --run option to call self and pipe )
- [] NixOS support via flakes/nix-shell
- [x] Add language of command and if it's installed
    - [x] If not installed, print a warning and link to Install page

## Basic Language Implementation
- [x] Python
- [x] Javascript
- [x] Bash
- [x] Shell
- [x] Rust
- [] Nushell
- [] Fish
- [] Typescript
- [] C
- [] C++
- [] C#
- [] Java

## Advanced Language Features
- [] Python
    - [] Pip dependencies
- [] Rust
    - [] Cargo dependencies
- [] Javascript
    - [] Npm dependencies


## Experimentation
- [] Use `git bash` on Windows when language is `sh/bash`

## Testing
- [x] Very very long file