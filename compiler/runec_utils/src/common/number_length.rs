/// Returns the number of digits in the number `n`.
///
/// # Example
/// ```
/// use runec_utils::common::number_length::number_length;
///
/// let len = number_length(12345);
/// assert_eq!(len, 5);
/// ```
pub fn number_length(n: usize) -> usize {
    n.checked_ilog10().map_or(1, |x| x as usize + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_length_handles_zero() {
        assert_eq!(number_length(0), 1);
    }

    #[test]
    fn number_length_handles_powers_of_ten() {
        assert_eq!(number_length(9), 1);
        assert_eq!(number_length(10), 2);
        assert_eq!(number_length(99), 2);
        assert_eq!(number_length(100), 3);
    }
}
