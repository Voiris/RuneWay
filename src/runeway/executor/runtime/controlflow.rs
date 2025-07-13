use super::types::RNWObjectRef;

#[derive(Debug, Clone)]
pub enum ControlFlow {
    Break,
    Continue,
    Nothing,
    Return(RNWObjectRef),
}
