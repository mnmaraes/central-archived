pub trait Storable {
    type IdType;
}

pub trait Storage {}

pub enum Stored<'s, T: Storable, S: Storage> {
    Pointer(T::IdType, &'s S),
    Value(T),
    Error(String),
}

pub mod async_pg;
