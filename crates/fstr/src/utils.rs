use std::ops::RangeBounds;

#[inline]
pub fn bounds_to_range(bounds: impl RangeBounds<usize>, len: usize) -> (usize, usize) {
    use std::ops::Bound;
    let begin = match bounds.start_bound() {
        Bound::Included(&n) => n,
        Bound::Excluded(&n) => n + 1,
        Bound::Unbounded => 0,
    };

    let end = match bounds.end_bound() {
        Bound::Included(&n) => n + 1,
        Bound::Excluded(&n) => n,
        Bound::Unbounded => len,
    };
    (begin, end)
}

#[inline(always)]
pub fn valid_str(s: &str, start: usize, end: usize) -> bool {
    s.get(start..end).is_some()
}