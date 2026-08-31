/// Push-only wrapper around `Vec<T>`.
///
/// Ensures that indices into the vector remain valid for the entire lifetime of
/// the vector
pub struct GrowingVec<T>(Vec<T>);

impl<T> GrowingVec<T> {
    pub fn push(&mut self, value: T) {
        self.0.push(value)
    }

    /// Provides read-only slice
    pub fn as_slice(&self) -> &[T] {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<T> From<Vec<T>> for GrowingVec<T> {
    fn from(vec: Vec<T>) -> Self {
        GrowingVec(vec)
    }
}

impl<T> FromIterator<T> for GrowingVec<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        GrowingVec(iter.into_iter().collect())
    }
}

impl<T> Default for GrowingVec<T> {
    fn default() -> Self {
        GrowingVec(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::GrowingVec;

    #[test]
    fn converts_from_vec() {
        let values = GrowingVec::from(vec![1, 2, 3]);

        assert_eq!(values.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn collects_from_iterator() {
        let values: GrowingVec<_> = (1..=3).collect();

        assert_eq!(values.as_slice(), &[1, 2, 3]);
    }
}
