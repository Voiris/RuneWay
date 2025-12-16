/// Push-only wrapper around `Vec<T>`.
///
/// Ensures that indices into the vector remain valid for the entire lifetime of the vector
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

    pub fn from_vec(vec: Vec<T>) -> Self {
        GrowingVec(vec)
    }
}

impl<T> Default for GrowingVec<T> {
    fn default() -> Self {
        GrowingVec(Vec::new())
    }
}
