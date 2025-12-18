/// Returns the number of digits in the number `n`.
///
/// Converts `n` to `f32`, takes the base-10 logarithm, rounds down (`floor`),
/// and adds 1 to get the number of digits.
///
/// # Example
/// ```
/// use runec_utils::common::number_length::number_length;
///
/// let len = number_length(12345);
/// assert_eq!(len, 5);
/// ```
pub fn number_length(n: usize) -> usize {
    (n as f32).log10().floor() as usize + 1
}