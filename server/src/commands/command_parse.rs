use crate::commands::string_reader::StringReader;
use anyhow::bail;
use std::str::FromStr;

pub trait CommandParse<'a>: Sized {
    fn parse(reader: &'a mut StringReader<'a>) -> anyhow::Result<Self>;
}

impl<'a> CommandParse<'a> for i32 {
    fn parse(reader: &'a mut StringReader<'a>) -> anyhow::Result<Self> {
        let str = reader.read_word();
        let int = Self::from_str(str)?;
        Ok(int)
    }
}

impl<'a> CommandParse<'a> for &'a str {
    fn parse(reader: &'a mut StringReader<'a>) -> anyhow::Result<Self> {
        let s = reader.read_word();
        if s.is_empty() {
            bail!("empty string")
        }
        Ok(s)
    }
}

pub struct GreedyString<'a> {
    pub str: &'a str
}

impl<'a> CommandParse<'a> for GreedyString<'a> {
    fn parse(reader: &'a mut StringReader<'a>) -> anyhow::Result<Self> {
        reader.skip_whitespace();
        Ok(GreedyString { str: reader.remaining() })
    }
}