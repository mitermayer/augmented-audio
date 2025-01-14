#[macro_export]
macro_rules! string_vec {
    ($($x:expr),*) => (vec![$($x.to_string()), *])
}

pub fn get_version() -> String {
    format!(
        "{}-{}-{}",
        env!("PROFILE"),
        env!("CARGO_PKG_VERSION"),
        env!("GIT_REV_SHORT")
    )
}
