#![allow(private_bounds)]

use crate::{shared::Shared, Fstr};

pub struct FstrIter<'a, T: Shared + ?Sized, I: Iterator<Item = &'a str>> {
    string: &'a T,
    iter: I,
}

// FstrIters should never be made with a iterator over a &str other than the Fstr's own.

impl<'a, T: Shared + ?Sized, I: Iterator<Item = &'a str>> FstrIter<'a, T, I> {
    #[inline(always)]
    pub fn new(string: &'a T, f: fn(&'a str) -> I) -> Self {
        Self { string, iter: f(string.as_str()) }
    }
    
    #[inline(always)]
    pub fn new_param<P>(string: &'a T, p: P, f: fn(&'a str, P) -> I) -> Self {
        Self { string, iter: f(string.as_str(), p) }
    }
}

impl<'a, T: Shared, I: Iterator<Item = &'a str>> Iterator for FstrIter<'a, T, I> {
    type Item = Fstr<'a>;
    
    // we can just piggy back off the input iterator and get our cheap copy strings out of it.
    // might be faster to not do that but it doesnt copy or allocate or anything i think so its fine.
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.iter.next()?;
        Some(self.string.recover(next))
    }
}