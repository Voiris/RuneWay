#[macro_export]
macro_rules! hashmap {
    ($( <$K:ty, $V:ty> )? $( $key:expr => $value:expr ),* $(,)? ) => {{
        let mut map $(: ::std::collections::HashMap<$K, $V>)? = ::std::collections::HashMap::new();
        $(
            map.insert($key, $value);
        )*
        map
    }};
}
