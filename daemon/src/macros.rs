#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        let dft = Local::now().to_string();
        println!("[INFO]\t{}\t{}", dft, format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        let dft = Local::now().to_string();
        eprintln!("[ERROR]\t{}\t{}", dft, format_args!($($arg)*));
    };
}
