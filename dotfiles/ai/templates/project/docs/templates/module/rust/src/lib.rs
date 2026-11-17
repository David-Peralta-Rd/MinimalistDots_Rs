pub fn hello() -> &'static str {
    "hello from __MODULE_NAME__"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_works() {
        assert_eq!(hello(), "hello from __MODULE_NAME__");
    }
}
