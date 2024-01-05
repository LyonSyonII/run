use std::{ops::Deref, collections::{HashSet, HashMap}};
mod parser;

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.iter().any(|arg| matches!(arg.as_str(), "-h" | "--help")) {
        println!("run: Runs a script.sh file in the current directory.");
        return;
    }
    
    let Ok(runfile) = std::fs::read_to_string("runfile") else { println!("run: runfile not found"); return };
    let runfile = parser::runfile::parse(runfile.deref()).expect("Could not parse runfile");
    
    match args.first().and_then(|c| runfile.commands.get(c.as_str())) {
        Some((cmdargs, script)) => run_command(args.get(1..).unwrap_or_default(), cmdargs, script),
        None => {
            let (cmdargs, script) = runfile.commands.get("default").goodbye("Could not find default command");
            run_command(args, cmdargs, script);
        },
    }
}

fn run_command<'a>(args: impl AsRef<[String]>, cmdargs: impl AsRef<[&'a str]>, script: &'a str) {
    let args = args.as_ref();
    let cmdargs = cmdargs.as_ref();
    
    if cmdargs.len() != args.len() {
        println!("run: Expected {cmdargs:?}, got {args:?}");
        std::process::exit(1);
    }
    let mut script = script.to_string();
    for (name, arg) in cmdargs.iter().zip(args.iter()) {
        let name = format!("${name}");
        script = script.replace(&name, arg);
    }
    std::process::Command::new("bash").arg("-c").arg(script).status().goodbye("Could not run script");   
}

pub struct Runfile<'i> {
    shebang: Option<&'i str>,
    commands: HashMap<&'i str, (Vec<&'i str>, &'i str)>
}


trait Goodbye<T> {
    fn goodbye(self, msg: impl AsRef<str>) -> T;
}

impl<T> Goodbye<T> for Option<T> {
    fn goodbye(self, msg: impl AsRef<str>) -> T {
        self.unwrap_or_else(|| { println!("run: {}", msg.as_ref()); std::process::exit(1) })
    }
}

impl<T, E> Goodbye<T> for Result<T, E> {
    fn goodbye(self, msg: impl AsRef<str>) -> T {
        self.unwrap_or_else(|_| { println!("run: {}", msg.as_ref()); std::process::exit(1) })
    }
}