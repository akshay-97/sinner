use crate::expressions::CqlType;

pub(crate) struct List<T>(T);

macro_rules! create_primitive_type {
    ($cql_type: ident) => {
        pub(crate) struct $cql_type();

        impl CqlType for $cql_type {}
    };
}

create_primitive_type!(Boolean);
create_primitive_type!(Int);
create_primitive_type!(BigInt);
create_primitive_type!(Text);
create_primitive_type!(Blob);
