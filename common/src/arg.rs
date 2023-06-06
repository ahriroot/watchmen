use clap::{Args, Parser, Subcommand};

// ========================== Watchmen ==========================
#[derive(Debug, Parser, PartialEq)]
#[command(author, about)]
pub struct TaskArgs {
    /// Config file path.
    /// Default: $HOME/.watchmen/config.toml
    #[arg(short, long)]
    pub config: Option<String>,

    /// Generate config file
    #[arg(short, long)]
    pub generate: Option<String>,

    /// Engine for send message
    #[arg(short, long, default_value = "sock")]
    pub engine: Option<String>,

    /// Start watchmen server
    #[arg(short, long)]
    pub daemon: bool,

    /// Start watchmen server with guard
    #[arg(short = 'w', long)]
    pub guard: Option<bool>,

    /// Print version
    #[arg(short, long)]
    pub version: bool,

    /// SubCommands
    #[command(subcommand)]
    pub commands: Option<Commands>,
}

impl TaskArgs {
    pub fn new() -> Self {
        Self::parse()
    }
}

#[derive(Debug, Subcommand, PartialEq)]
pub enum Commands {
    /// Add and run a task
    Run(AddArgs),
    Add(AddArgs),
    Start(FlagArgs),
    Stop(FlagArgs),
    Remove(FlagArgs),
    List(FlagArgs),
}

#[derive(Args, Debug, PartialEq)]
pub struct FlagArgs {
    /// Task config directory
    #[arg(short = 'p', long)]
    pub path: Option<String>,

    /// Task config directory
    #[arg(short = 'm', long, default_value = r"^.*\.(toml|ini|json)$")]
    pub mat: Option<String>,

    /// Task config file
    #[arg(short = 'f', long)]
    pub config: Option<String>,

    /// Task name (unique)
    #[arg(short, long)]
    pub name: Option<String>,
}

#[derive(Args, Debug, PartialEq)]
pub struct AddArgs {
    /// Task config directory
    #[arg(short = 'p', long)]
    pub path: Option<String>,

    /// Task config directory
    #[arg(short = 'm', long, default_value = r"^.*\.(toml|ini|json)$")]
    pub mat: Option<String>,

    /// Task config file
    #[arg(short = 'f', long)]
    pub config: Option<String>,

    /// Task name (unique)
    #[arg(short, long)]
    pub name: Option<String>,

    /// Task command
    #[arg(short, long)]
    pub command: Option<String>,

    /// Task arguments
    #[arg(short, long)]
    pub args: Option<Vec<String>>,

    /// Task working directory
    #[arg(short = 'd', long)]
    pub dir: Option<String>,

    /// Task environment variables
    #[arg(short = 'e', long)]
    pub env: Option<Vec<String>>,

    /// Task standard input
    #[arg(short = 'i', long)]
    pub stdin: Option<String>,

    /// Task standard output
    #[arg(short = 'o', long)]
    pub stdout: Option<String>,

    /// Task standard error
    #[arg(short = 'r', long)]
    pub stderr: Option<String>,
}

// ========================== Daemon ==========================
#[derive(Debug, Parser, PartialEq)]
#[command(author, about)]
pub struct DaemonArgs {
    /// Config file path.
    /// Default: $HOME/.watchmen/config.toml
    #[arg(short, long)]
    pub config: Option<String>,

    /// Print version
    #[arg(short, long)]
    pub version: bool,
}

impl DaemonArgs {
    pub fn new() -> Self {
        Self::parse()
    }
}
