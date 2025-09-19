#[macro_export]
macro_rules! raw_format {
    () => {};
}

#[macro_export]
macro_rules! raw_println {
    () => {
        println!("\r");
    };
    ($($arg:tt)*) => {{
        print!($($arg)*);
        raw_println!();
    }};
}

#[macro_export]
macro_rules! raw_dbg {
    () => {
        raw_println!(
            "[{}:{}:{}]",
            file!(),
            line!(),
            column!()
        )
    };

    ($val:expr $(,)?) => {
        match $val {
            tmp => {
                raw_println!(
                    "[{}:{}:{}] {} = {:?}",
                    file!(),
                    line!(),
                    column!(),
                    stringify!($val),
                    &&tmp as &dyn std::fmt::Debug,
                );
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($(raw_dbg!($val)),+,)
    };
}
