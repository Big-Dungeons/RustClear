

pub struct StringReader<'a> {
    pub str: &'a str,
    cursor: usize,
}

impl<'a> StringReader<'a> {

    pub fn new(str: &'a str) -> Self {
        Self {
            str,
            cursor: 0,
        }
    }

    pub fn remaining(&self) -> &'a str {
        &self.str[self.cursor..]
    }

    pub fn peek(&self) -> Option<char> {
        self.remaining().chars().next()
    }

    pub fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.cursor += ch.len_utf8();
        Some(ch)
    }

    pub fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(c) if c.is_whitespace()) {
            self.advance();
        }
    }

    pub fn read_word(&mut self) -> &'a str {
        self.skip_whitespace();
        let start = self.cursor;
        while matches!(self.peek(), Some(c) if !c.is_whitespace()) {
            self.advance();
        }
        &self.str[start..self.cursor]
    }

    pub fn read_rest(&mut self) -> &'a str {
        self.skip_whitespace();
        let start = self.cursor;
        self.cursor = self.str.len();
        &self.str[start..]
    }
}