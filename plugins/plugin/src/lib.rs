use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[repr(C)]
pub struct Task {
    pub id: u128,
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub status: String,
    pub pid: u32,
    pub created_at: u128,
    pub started_at: u128,
    pub exited_at: u128,
    pub stopped_at: u128,
    pub laststart_at: u128,
    pub exit_code: u32,
    pub interval: u128,
    pub origin: u128,
    pub timing: Vec<u128>,
}

#[no_mangle]
pub async extern "C" fn func_plugin(t: i32) -> bool {
    print!("PRINT FROM PLUGIN: {:?}", t);
    false
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
