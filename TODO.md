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
- [x] C#
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

## Fixes
- [x] Fix `run -- <ARGS>` to passthrough arguments to the command when the default command
     does not have any arguments
        - [x] Command output is being piped into "/tmp/run/input/input"

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