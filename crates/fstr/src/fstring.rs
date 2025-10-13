use std::{borrow::Borrow, fmt::Display, hash::Hash, ops::RangeBounds, str::Utf8Error};

use uuid::{Uuid, fmt::Hyphenated};

use crate::{import_shared, inner::Inner, shared::Shared, Fstr};

/// Cheaply clonable, immutable string, with the same stack size as a String.
/// FStrings have substrings similar to bytes::Bytes, which
/// enables iterators over FStrings to produce references to the same allocation but over different parts.
/// 
/// inlined if the string is less than 22 bytes,
/// Arced if more,
/// Static if static.
/// 
/// FStrings must be explicitely created to and from string slices to ensure allocations are explicit. 
/// if you dont need an owned slice but may pass it around somewhere that does, you can use an Fstr or &FString.
/// &'static strs can be made using FString::from() and will not allocate.
/// non static strs can be made into FStrings using either FString::new() or str.to_fstring()
/// 
/// FStrings do not implicitely deref into strs to prevent str view methods (ex: most str iterators) from being used over FString versions.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct FString(pub(super) Inner);

impl FString {
    pub const EMPTY: FString = FString(Inner::Static(""));
    
    /// creates an fstring from a static reference.
    /// 
    /// this will never allocate.
    #[inline(always)]
    pub fn new_static(s: &'static str) -> Self {
        Self(Inner::Static(s))
    }
    
    /// Creates a FString from a string reference.
    /// 
    /// This will allocate if the str is longer than 22 bytes.
    #[inline(always)]
    pub fn new(s: &str) -> Self {
        Self(Inner::new(s))
    }
    
    /// Creates a Fstr from this FString.
    /// 
    /// This will not clone, copy data, or allocate.
    #[inline(always)]
    pub fn as_fstr(&self) -> Fstr<'_> {
        Fstr::from_fstring(self, ..)
    }
    
    /// creates a new FString from bytes.
    /// This will fail if the bytes are not valid UTF-8.
    /// 
    /// This will allocate if the byte slice contains more than 22 bytes.
    #[inline(always)]
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Utf8Error> {
        let str = str::from_utf8(bytes)?;
        Ok(Self::new(str))
    }
    
    /// gets the length of the string.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.0.len()
    }
    
    /// returns true if the string is stored inline on the stack.
    #[inline(always)]
    pub fn is_inline(&self) -> bool {
        matches!(self.0, Inner::Inline { .. })
    }
    
    import_shared!();
}

impl Shared for FString {
    #[inline(always)]
    fn substr(&self, range: impl RangeBounds<usize>) -> Fstr<'_> {
        Fstr::from_fstring(self, range)
    }
    
    #[inline]
    fn recover(&self, str: &str) -> Fstr<'_> {
        let self_ptr = self.as_str().as_ptr() as usize;
        let slice_ptr = str.as_ptr() as usize;
        
        let start = slice_ptr - self_ptr;
        let end = start + str.len();
        
        self.substr(start..end)
    }
    
    #[inline(always)]
    fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<&Fstr<'_>> for FString {
    fn from(value: &Fstr<'_>) -> Self {
        value.to_owned()
    }
}

impl From<&Uuid> for FString {
    fn from(value: &Uuid) -> Self {
        FString::new(value.as_hyphenated().encode_lower(&mut Uuid::encode_buffer()))
    }
}

impl From<Uuid> for FString {
    fn from(value: Uuid) -> Self {
        FString::new(value.hyphenated().encode_lower(&mut Uuid::encode_buffer()))
    }
}

impl From<Fstr<'_>> for FString {
    fn from(value: Fstr<'_>) -> Self {
        value.to_owned()
    }
}

impl From<&FString> for FString {
    #[inline(always)]
    fn from(value: &FString) -> Self {
        value.clone()
    }
}

impl From<&String> for FString {
    #[inline(always)]
    fn from(value: &String) -> Self {
        Self::new(value)
    }
}

impl From<&'static str> for FString {
    #[inline(always)]
    fn from(value: &'static str) -> Self {
        Self::new_static(value)
    }
}

impl Borrow<str> for FString {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Display for FString {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.as_str(), f)
    }
}