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

    /// Print version
    #[arg(short, long)]
    pub version: bool,

    /// Sub Commands
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
    /// Add and run tasks
    Run(AddArgs),
    /// Add tasks
    Add(AddArgs),
    /// Reload tasks
    Reload(AddArgs),
    /// Start tasks
    Start(FlagArgs),
    /// Restart tasks
    Restart(FlagArgs),
    /// Stop tasks
    Stop(FlagArgs),
    /// Remove tasks
    Remove(FlagArgs),
    /// Pause interval tasks
    Pause(FlagArgs),
    /// Resume interval tasks
    Resume(FlagArgs),
    /// Get tasks list
    List(ListArgs),
}

#[derive(Args, Debug, PartialEq)]
pub struct ListArgs {
    /// Task config directory
    #[arg(short = 'p', long)]
    pub path: Option<String>,

    /// Task config filename regex pattern
    #[arg(short = 'r', long, default_value = r"^.*\.(toml|ini|json)$")]
    pub regex: Option<String>,

    /// Task config file
    #[arg(short = 'f', long)]
    pub config: Option<String>,

    ///  Task id (unique)
    #[arg(short = 'i', long)]
    pub id: Option<i64>,

    /// Task name (unique)
    #[arg(short = 'n', long)]
    pub name: Option<String>,

    /// Task group
    #[arg(short, long)]
    pub group: Option<String>,

    /// Is match regex pattern by name
    #[arg(short = 'm', long)]
    pub mat: bool,

    /// Show more info
    #[arg(short = 'o', long, default_value = "false")]
    pub more: bool,

    /// Show less info
    #[arg(short = 'l', long, default_value = "false")]
    pub less: bool,
}

#[derive(Args, Debug, PartialEq)]
pub struct FlagArgs {
    /// Task config directory
    #[arg(short = 'p', long)]
    pub path: Option<String>,

    /// Task config filename regex pattern
    #[arg(short = 'r', long, default_value = r"^.*\.(toml|ini|json)$")]
    pub regex: Option<String>,

    /// Task config file
    #[arg(short = 'f', long)]
    pub config: Option<String>,

    /// Task id (unique)
    #[arg(short, long)]
    pub id: Option<i64>,

    /// Task name (unique)
    #[arg(short, long)]
    pub name: Option<String>,

    /// Task group
    #[arg(short, long)]
    pub group: Option<String>,

    /// Is match regex pattern by namae
    #[arg(short = 'm', long)]
    pub mat: bool,
}

#[derive(Args, Debug, PartialEq)]
pub struct AddArgs {
    /// Task config directory
    #[arg(short = 'p', long)]
    pub path: Option<String>,

    /// Task config filename regex pattern
    #[arg(short = 'r', long, default_value = r"^.*\.(toml|ini|json)$")]
    pub regex: Option<String>,

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

    /// Task group
    #[arg(short, long)]
    pub group: Option<String>,

    /// Task working directory
    #[arg(short = 'd', long)]
    pub dir: Option<String>,

    /// Task environment variables
    #[arg(short = 'e', long)]
    pub env: Option<Vec<String>>,

    /// Task standard input
    #[arg(short = 'i', long, default_value = "false")]
    pub stdin: bool,

    /// Task standard output
    #[arg(short = 'o', long)]
    pub stdout: Option<String>,

    /// Task standard error
    #[arg(short = 'w', long)]
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

    /// Load cached tasks
    #[arg(short, long, default_value = "true")]
    pub load: bool,

    /// Print version
    #[arg(short, long)]
    pub version: bool,
}

impl DaemonArgs {
    pub fn new() -> Self {
        Self::parse()
    }
}
