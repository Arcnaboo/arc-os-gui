pub trait Agent {
    fn process(&self, input: &str) -> String;
}
