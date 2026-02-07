use crate::commands::string_reader::StringReader;
use std::str::FromStr;

pub trait CommandParse: Sized {
    fn parse(reader: &mut StringReader) -> anyhow::Result<Self>;
}

impl CommandParse for i32 {
    fn parse(reader: &mut StringReader) -> anyhow::Result<Self> {
        let str = reader.read_word();
        let int = Self::from_str(str)?;
        Ok(int)
    }
}
