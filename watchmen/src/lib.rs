pub mod command;
pub mod entity;
pub mod socket;

pub mod const_exit_code {
    pub enum ExitCode {
        SUCCESS = 0,
        ERROR = 1,
    }
}
