use crate::cql::*;

pub trait CqlType {}

///
/// AsExpression is used to statically verify if the Rust type is compatible
/// with CQL types or not
///
pub trait AsExpression<T: CqlType> {}

macro_rules! impl_as_expression {
    ($cql_type: ty, $rust_type: ty) => {
        impl AsExpression<$cql_type> for $rust_type {}
    };
}

macro_rules! impl_as_expression_generic {
    ($cql_type: ty, $rust_type: ty) => {
        impl<T: CqlType> AsExpression<$cql_type> for $rust_type where $cql_type: CqlType {}
    };
}

impl_as_expression!(Boolean, bool);
impl_as_expression!(Int, i32);
impl_as_expression!(Int, u32);
impl_as_expression!(BigInt, i64);
impl_as_expression!(BigInt, u64);
impl_as_expression!(Text, String);
impl_as_expression!(Blob, Vec<u8>);
impl_as_expression_generic!(List<T>, Vec<T>);
