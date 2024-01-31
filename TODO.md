# TODO

## Features
- [x] Constants
- [] Optional arguments
- [] Command alias
- [] --run <LANG\> option, runs stdin as that language
- [] Run commands from languages 
  - [] Shell/Bash (use --run option to call self and pipe )
- [x] NixOS support via flakes/nix-shell
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
- [x] C
- [x] C++
- [] C#
- [] Java

## Advanced Language Features
- [] Python
    - [] Pip dependencies
- [] Rust
    - [] Cargo dependencies
- [] Javascript
    - [] Npm dependencies
- [] C#
    - [] Nuget dependencies?

## Documentation
- [] Add basic README with at least a copy of the --help menu


## Experimentation
- [] Use `git bash` on Windows when language is `sh/bash`

## Code Improvement
- [] Use common "create_project" for all languages (based on cpp.rs)
- [] Do not capture command output (inherit instead) and write to tmp file always (instead of using stdin)

## Testing
- [x] Very very long file