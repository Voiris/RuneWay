mod integer;
mod float;
mod null_type;
mod string;
mod boolean;
mod list;
mod iterator;

pub use integer::RNWInteger;
pub use float::RNWFloat;
pub use null_type::RNWNullType;
pub use string::RNWString;
pub use boolean::RNWBoolean;
pub use list::RNWList;
pub use iterator::RNWIterator;
use crate::runeway::executor::runtime::types::register_type;

pub fn register_basic_types() {
    register_type::<RNWNullType>(RNWNullType::type_name());
    register_type::<RNWInteger>(RNWInteger::type_name());
    register_type::<RNWString>(RNWString::type_name());
    register_type::<RNWBoolean>(RNWBoolean::type_name());
    register_type::<RNWList>(RNWList::type_name());
    register_type::<RNWFloat>(RNWFloat::type_name());
    register_type::<RNWIterator>(RNWIterator::type_name());
}
