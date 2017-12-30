pub trait Writer {
    fn append(&mut self, string: &str);
}
