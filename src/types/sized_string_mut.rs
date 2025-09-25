use crate::types::sized_string::SizedString;

// essentially a builder for sized string

#[repr(transparent)]
pub struct SizedStringMut<const S: usize> {
    pub(super) inner: SizedString<S>
}

impl<const S: usize> SizedStringMut<S> {
    
    pub fn new() -> Self {
        Self {
            inner: SizedString::EMPTY,
        }
    }
    
    pub fn from(string: impl Into<SizedString<S>>) -> Self {
        Self {
            inner: string.into(),
        }
    }
    
    pub fn append_char(&mut self, char: char) -> &mut Self {
        let inner = &mut self.inner;
        let len = char.len_utf8();
        assert!(
            inner.length + len <= S,
            "tried appending a char into a full sized string, len: {}, char len: {len}", self.inner.length
        );
        match len { 
            1 => inner.data[inner.length] = char as u8,
            _ => {
                let mut arr = [0; 4];
                let encoded = char.encode_utf8(&mut arr);
                let bytes = encoded.as_bytes();
                inner.data[inner.length..inner.length + bytes.len()].copy_from_slice(bytes);
            }
        }
        inner.length += len;
        self
    }
    
    pub fn append_str(&mut self, str: &str) -> &mut Self {
        let inner = &mut self.inner;
        let bytes = str.as_bytes();
        assert!(
            inner.length + bytes.len() <= S,
            "tried appending a str into a full sized string, len: {}, char len: {}", self.inner.length, bytes.len(),
        );
        inner.data[inner.length..inner.length + bytes.len()].copy_from_slice(&bytes);
        inner.length += bytes.len();
        self
    }
    
}