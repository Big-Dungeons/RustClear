use std::ops::RangeBounds;

use crate::{Fstr, FstrIter};

pub(super) trait Shared {
    fn recover(&self, str: &str) -> Fstr<'_>;
    fn substr(&self, range: impl RangeBounds<usize>) -> Fstr<'_>;
    
    fn with_iter<'a, I: Iterator<Item = &'a str>>(&'a self, iter: fn(&'a str) -> I) -> FstrIter<'a, Self, I> {
        FstrIter::new(self, iter)
    }
    
    fn with_iter_param<'a, P, I: Iterator<Item = &'a str>>(&'a self, p: P, iter: fn(&'a str, P) -> I) -> FstrIter<'a, Self, I> {
        FstrIter::new_param(self, p, iter)
    }
    
    fn as_str(&self) -> &str;
}

#[macro_export]
macro_rules! import_shared {
    () => {
        /// Recovers a Fstr from this &str if it is a reference to the same data.
        /// 
        /// This will never allocate.
        #[inline(always)]
        pub fn recover(&self, s: &str) -> Fstr<'_> {
            <Self as Shared>::recover(self, s)
        }

        /// Gets a Fstr reference to a substring of this Fstr/FString.
        /// 
        /// This will never allocate.
        #[inline(always)]
        pub fn substr(&self, range: impl std::ops::RangeBounds<usize>) -> Fstr<'_> {
            <Self as Shared>::substr(self, range)
        }

        /// Gets an iterator over this Fstr/FString using a str iterator.
        /// 
        /// strs the backing iterator would normally return are recovered into Fstrs for cheap cloning.
        #[inline(always)]
        pub fn with_iter<'l, I: Iterator<Item = &'l str>>(
            &'l self,
            iter: fn(&'l str) -> I,
        ) -> super::FstrIter<'l, Self, I>
        where
            Self: Sized,
        {
            <Self as Shared>::with_iter(self, iter)
        }
        
        /// Gets an iterator with a single parameter over this Fstr/FString using a str iterator.
        /// 
        /// strs the backing iterator would normally return are recovered into Fstrs for cheap cloning.
        #[inline(always)]
        pub fn with_iter_param<'l, P, I: Iterator<Item = &'l str>>(
            &'l self,
            p: P,
            iter: fn(&'l str, P) -> I,
        ) -> super::FstrIter<'l, Self, I>
        where
            Self: Sized,
        {
            <Self as Shared>::with_iter_param(self, p, iter)
        }

        /// gets a &str from this Fstr/FString.
        #[inline(always)]
        pub fn as_str(&self) -> &str {
            <Self as Shared>::as_str(self)
        }
    };
}