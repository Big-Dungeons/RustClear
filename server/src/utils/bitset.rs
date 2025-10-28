/// const bitset implementation.
/// The size of the bitset must be ceilinged (max value + 1) / 64, IE: a max value of 127 would be size 2, 128 size 3, 129 size 3, etc.
///
/// # Example
/// ```
/// const BITSET: BitSet<2> = BitSet::<2>::new(&[0, 3, 5, 7, 64, 126]);
///
/// fn main() {
///     assert_eq!(BITSET.contains(0), true);
///     assert_eq!(BITSET.contains(2), false);
///     assert_eq!(BITSET.contains(126), true);
///     assert_eq!(BITSET.contains(134), false);
///
///     let mut block = String::new();
///
///     for (index, word) in BITSET.0.iter().enumerate() {
///         for i in 0..64 {
///             let bit = (word >> i) & 1;
///             block.push_str(&bit.to_string());
///         }
///         if index == 0 {
///             assert_eq!(block, "1001010100000000000000000000000000000000000000000000000000000000")
///         }
///         if index == 1 {
///             assert_eq!(block, "1000000000000000000000000000000000000000000000000000000000000010")
///         }
///         block.clear();
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct BitSet<const N: usize>([u64; N]);

impl<const N: usize> BitSet<N> {
    /// Creates a new bitset given the input values.
    /// 
    /// Panics if a value exceeds the maximum supported by the size of the bitset.
    pub const fn new(values: &[usize]) -> Self {
        let mut bits = [0u64; N];
        let mut i = 0;
        while i < values.len() {
            let bit = values[i];
            bits[bit / 64] |= 1 << (bit % 64);
            i += 1;
        }
        Self(bits)
    }

    pub const fn contains(&self, bit: usize) -> bool {
        if bit >= 64 * N { return false; }
        self.0[bit / 64] & (1 << (bit % 64)) != 0
    }
}