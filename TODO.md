# TODO

## Features
- [] Optional/default arguments
- [] Command alias
- [] --run <LANG\> option, runs stdin as that language
- [] Run commands from languages 
  - [] Shell/Bash (use --run option to call self and pipe)
- [] Include external file
- [x] Constants
- [x] NixOS support via flakes/nix-shell
- [x] Add language of command and if it's installed
    - [x] If not installed, print a warning and link to Install page


## Basic Language Implementation
- [] Nushell
- [] Fish
- [] Typescript
- [] Java
- [x] Python
- [x] Javascript
- [x] Bash
- [x] Shell
- [x] Rust
- [x] C
- [x] C++
- [x] C#

## Advanced Language Features
- [] Python
    - [] Pip dependencies
- [] Rust
    - [] Cargo dependencies
    - [] Include external file
- [] Javascript
    - [] Npm dependencies
- [] C#
    - [] Nuget dependencies?

## Documentation
- [] Add possible arguments to README
- [] Add examples to README
- [x] Specify that -f must be the first argument


## Experimentation
- [] Use `git bash` on Windows when language is `sh/bash`

## Code Improvement
- [x] Use common "create_project" for all languages (based on cpp.rs)
- [x] Do not capture command output (inherit instead) and write to tmp file always (instead of using stdin)
- [] Improve error message when too many args are passed

## Testing
- [x] Very very long file