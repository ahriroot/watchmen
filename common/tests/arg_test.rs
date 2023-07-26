#[cfg(test)]
mod tests {
    use clap::Parser;
    use common::arg::{AddArgs, Commands, TaskArgs};

    #[test]
    fn test_arg() {
        let args = TaskArgs::parse_from(["--", "-v"]);
        assert!(args.version);

        let args = TaskArgs::parse_from(["--", "run", "-n", "test", "-c", "rustc"]);
        let cmd = Commands::Run(AddArgs {
            path: None,
            regex: None,
            config: None,
            name: None,
            command: None,
            args: None,
            dir: None,
            env: None,
            stdin: false,
            stdout: None,
            stderr: None,
        });
        assert_eq!(args.commands, Some(cmd));
    }
}
