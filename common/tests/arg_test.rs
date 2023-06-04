#[cfg(test)]
mod tests {
    use clap::Parser;
    use common::arg::{Commands, RunArgs, TaskArgs};

    #[test]
    fn test_arg() {
        let args = TaskArgs::parse_from(["--", "-v"]);
        assert!(args.version);

        let args = TaskArgs::parse_from(["--", "run", "-n", "test", "-c", "rustc"]);
        let cmd = Commands::Run(RunArgs {
            all: None,
            config: None,
            name: "test".to_string(),
            command: "rustc".to_string(),
            args: None,
            dir: None,
            env: None,
            stdin: None,
            stdout: None,
            stderr: None,
        });
        assert_eq!(args.commands, Some(cmd));
    }
}
