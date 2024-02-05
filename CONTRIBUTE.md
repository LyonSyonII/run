# Contribute

## Adding a new language

### Issue
If you don't know Rust and want to request a new language, you can do so by creating a new issue with the title `[Add Language] LANGUAGE` and with the following information:
```yaml
- name: LANGUAGE
  # the names valid in the runfile
- aliases:
  - ALIAS1
  - ALIAS2
  # the name of the binary needed to run the code
- binary: LANG_COMPILER
  # nix packages needed to run the code
  # https://search.nixos.org/packages?channel=unstable
- nix-packages: 
  - LANG_PACKAGE1
  - LANG_PACKAGE2
  # steps to execute the code (from project creation to binary execution)
- execute:      
  - init:     # steps to initialize project, can be empty
  - compile:  # steps to compile the code, can be empty
  - run:      # steps to run the code
  # binaries needed for the language to work
- installed:
  - LANG_BINARY1
  - LANG_BINARY2
  - LANG_BINARY3
```

### Pull Request
If you know Rust and want to add a new language, you can do so by following these steps:
- Clone the repository `git clone https://github.com/lyonsyonii/run`
- Create a new branch `git checkout -b add-lang-LANGUAGE`
- Go to `cli/src/lang/` and create a new file `LANGUAGE.rs`
- Copy the following code into the file and replace as necessary:
    ```rust
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct LANGUAGE;

    impl super::Language for LANGUAGE {
        fn as_str(&self) -> &'static str {
            // will be displayed in the help message
            "LANG" 
        }

        fn binary(&self) -> &'static str {
            // the name of the binary needed to run the code
            "LANG_COMPILER"
        }

        fn nix_packages(&self) -> &'static [&'static str] {
            // the nix packages needed to run the code
            &["LANG_PACKAGE1", "LANG_PACKAGE2"]
        }
        
        fn execute(&self, input: &str, args: impl AsRef<[String]>) -> Result<(), Str<'_>> {
            // steps to execute the code
            // you can use super::execute_interpreted or super::execute_compiled
            execute_interpreted(self.program()?, input, args)
        }

        fn installed(&self) -> bool {
            // if only `self.binary()` is needed, you can use the default implementation
            // if multiple binaries are needed, use
            super::installed_all(&[self.binary(), "LANG_BINARY2", "LANG_BINARY3"])
            // if multiple alternatives exist, use
            super::installed_any(&["LANG_BINARY1", "LANG_BINARY2", "LANG_BINARY3"])
        }

        fn program(&self) -> Result<std::process::Command, Str<'static>> {
            // fetch the program to run the code
            // generally you can use the default implementation
        }
    }
    ```
- Add the new language to the enum in `cli/src/lang/mod.rs`
  ```rust
  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub enum Language {
    Shell,
    Bash,
    Rust,
    Python,
    Javascript,
    C,
    Cpp,
    CSharp,
    // add the new language here
    LANGUAGE,
  }
  ```
- Add the names of the language to the `match` statement in the implementation of `FromStr` in `cli/src/lang/mod.rs`
  ```rust
  match s {
      "cmd" | "fn" | "sh" | "shell" => Ok(Shell.into()),
      "bash" => Ok(Bash.into()),
      "rs" | "rust" => Ok(Rust.into()),
      "c" => Ok(C.into()),
      "c++" | "cpp" | "cplusplus" => Ok(Cpp.into()),
      "c#" | "cs" | "csharp" => Ok(CSharp.into()),
      "py" | "python" => Ok(Python.into()),
      "js" | "javascript" => Ok(Javascript.into()),
      // add the new language here
      "ALIAS1" | "ALIAS2" => Ok(LANGUAGE.into()),
      _ => Err(s.to_owned()),
  }
  ```
- Check if the code compiles and recognizes the new language
- Create a Pull Request with your changes and with title `[Add Language] LANGUAGE`