pub trait TokenReader {
    fn read_identifier(&mut self) -> String;
    fn read_number(&mut self) -> String;
    fn read_string(&mut self) -> String;
}