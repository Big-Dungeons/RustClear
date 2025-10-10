use std::{hash::Hash, ops::Range, str::FromStr};

use arcstr::{ArcStr, Substr};

use crate::utils::valid_str;
 
#[derive(Debug, Clone)]
pub(super) enum Inner {
    Inline {
        len: u8,
        bytes: [u8; 22],
    },
    Arced(ArcStr),
    Substr(Substr),
    Static(&'static str), // technically ArcStr can do static strings, but it doesnt give us a good method of handling them without a macro.
}

impl Inner {
    pub(super) fn new(s: &str) -> Self {
        if s.len() <= 22 {
            // SAFETY: the entire string will always be valid if its smaller than 22 bytes.
            unsafe {
                Self::inline_unchecked(s.as_bytes(), 0..s.len())
            }
        } else {
            Self::Arced(ArcStr::from_str(s).unwrap())
        }
    }
    
    #[inline(always)]  
    pub(super) fn len(&self) -> usize {
        match self {
            Self::Inline { len, bytes: _ } => *len as usize,
            Self::Arced(str) => str.len(),
            Self::Substr(str) => str.len(),
            Self::Static(str) => str.len(),
        }
    }
    
    #[inline(always)]
    pub(super) fn as_str(&self) -> &str {
        match self {
            Self::Inline { len, bytes } => unsafe {
                // SAFETY: FStringInners should always be valid on construction.
                str::from_utf8_unchecked(&bytes[..*len as usize])
            }
            Self::Arced(str) => str,
            Self::Substr(str) => str,
            Self::Static(str) => str,
        }
    }
    
    pub(super) unsafe fn inline_unchecked(bytes: &[u8], range: Range<usize>) -> Self {
        let mut buffer = [0u8; 22];
        debug_assert!(valid_str(str::from_utf8(bytes).unwrap(), range.start, range.end));
        let len = range.end - range.start;
        unsafe {
            buffer.get_unchecked_mut(..len).copy_from_slice(bytes.get_unchecked(range));
        }
        Self::Inline { len: len as u8, bytes: buffer }
    }
}

impl PartialEq for Inner {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for Inner {}

impl Hash for Inner {
    #[inline(always)]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}
