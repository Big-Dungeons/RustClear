use std::{fmt::Display, ops::RangeBounds, ptr::slice_from_raw_parts, slice};

use arcstr::Substr;

use crate::{import_shared, inner::Inner, shared::Shared, utils::{bounds_to_range, valid_str}, FString};

/// Reference into an owned FString.
/// Think of it as how &str is to String.
/// This doesnt increment the reference count until its converted to an owned value.
/// 
/// Fstrs do not implicitely deref into strs to prevent str view methods (ex: most str iterators) from being used over Fstr versions.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Fstr<'a> {
    string: &'a Inner,
    start: u32,
    end: u32,
}

impl<'a> Fstr<'a> {
    pub const EMPTY: Fstr<'static> = Fstr {
        string: &Inner::Static(""),
        start: 0,
        end: 0,
    };
    
    /// creates a new FStr from a FString given range bounds.
    /// 
    /// # Panics
    ///
    /// May panic if the index is out of bounds.
    pub fn from_fstring(s: &'a FString, range: impl RangeBounds<usize>) -> Self {
        let (start, end) = bounds_to_range(range, s.len());
        
        if valid_str(s.as_str(), start, end) {
            // SAFETY: We just checked that the indexes are valid for the substring.
            unsafe { Self::new_unchecked(&s.0, start, end) }
        } else {
            panic!("{}, index by {}..{} is invalid.", s.as_str(), start, end)
        }
    }

    pub(super) unsafe fn new_unchecked(s: &'a Inner, start: usize, end: usize) -> Self {
        Self {
            string: s,
            start: start as u32,
            end: end as u32,
        }
    }

    /// gets the length of the string.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.end as usize - self.start as usize
    }
    
    fn parent_len(&self) -> usize {
        self.string.len()
    }
    
    /// Creates an owned FString from this FStr.
    /// 
    /// This will inline itself if the str is less than 22 bytes,
    /// Increment the referene count if it is backed by an arc,
    /// or return a static reference if the underlying data is static.
    pub fn to_owned(&self) -> FString {
        let (start, end) = (self.start as usize, self.end as usize);
        debug_assert!(valid_str(self.string.as_str(), start, end), "Fstr_to_owned");
        
        if self.len() <= 22 {
            let bytes = self.string.as_str().as_bytes();
            // SAFETY: Fstrs should be valid on construction.
            let inner = unsafe { Inner::inline_unchecked(bytes, start..end) };
            return FString(inner)
        }
        
        let inner = match self.string {
            Inner::Inline { len: _, bytes: _ } => unreachable!(), // full len inline strs will just create a new inline from the check above.
            Inner::Arced(arc) => Inner::Substr(unsafe { Substr::from_parts_unchecked(arc.clone(), start..end) }),
            Inner::Substr(str) => {
                let range_start = str.range().start;
                let start = range_start + start;
                let end = range_start + end;
                Inner::Substr(unsafe { Substr::from_parts_unchecked(str.parent().clone(), start..end) })
            }
            Inner::Static(str) => Inner::Static(unsafe { &str.get_unchecked(start..end) })
        };
        
        FString(inner)
    }

    import_shared!();
}

impl Shared for Fstr<'_> {
    #[inline]
    fn substr(&self, range: impl RangeBounds<usize>) -> Fstr<'_> {
        let (range_start, range_end) = bounds_to_range(range,self.parent_len());
        let start = range_start as u32 + self.start;
        let end = (range_start + range_end) as u32;
        Self {
            string: self.string,
            start,
            end,
        }
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
        let (start, end) = (self.start as usize, self.end as usize);
        // SAFETY: Fstrs should be valid on construction.
        unsafe { self.string.as_str().get_unchecked(start..end) }
    }
}

impl Display for Fstr<'_> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.as_str(), f)
    }
}