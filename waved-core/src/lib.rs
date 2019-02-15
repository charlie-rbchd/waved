#[no_mangle]
pub fn get_message() -> &'static str {
    "Hello, Dylib"
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
