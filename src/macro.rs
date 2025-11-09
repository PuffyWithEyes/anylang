macro_rules! error {
    ($err:expr) => {
        format!("[{}:ERROR] {}", crate::CRATE_NAME, $err)
    };
}
