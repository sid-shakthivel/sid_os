// Holds a number of standard methods used to display text

pub trait Output {
    fn write_string(&mut self, string: &str) {
        for c in string.chars() {
            self.put_char(c);
        }
    }

    fn put_char(&mut self, character: char) {}
    fn clear(&mut self) {}
    fn new_line(&mut self) {}
}
