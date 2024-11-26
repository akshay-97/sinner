use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum CqlType {
    Str(String),
    Bool(bool),
    Row(CqlMap),
    NumInt(i64),
    NumFloat(f64),
    Timestamp(i64),
    Bytes(Vec<u8>),
    Null,
}

pub type CqlMap = HashMap<String, CqlType>;
pub type CqlMapWithQuery = (QueryString, HashMap<String, CqlType>);
type QueryString = String;

pub trait ToCqlData {
    fn to_cql(self) -> CqlType;
}

pub trait ToCqlRow {
    type Output;
    // fn to_row(self) -> CqlMap;
    //fn to_row_iter<T, S : Iterator<Item = T>>(self: dyn IntoIterator<Item = T, IntoIter = S>) -> impl IntoIterator;
    fn to_row_iter(self) -> impl Iterator<Item = Self::Output>;
}

impl ToCqlData for String {
    fn to_cql(self) -> CqlType {
        CqlType::Str(self.clone())
    }
}

impl ToCqlData for i64 {
    fn to_cql(self) -> CqlType {
        CqlType::NumInt(self.clone())
    }
}

impl ToCqlData for f64 {
    fn to_cql(self) -> CqlType {
        CqlType::NumFloat(self.clone())
    }
}

impl ToCqlData for time::OffsetDateTime {
    fn to_cql(self) -> CqlType {
        CqlType::Timestamp(self.unix_timestamp())
    }
}

impl FromCqlData for time::OffsetDateTime {
    type Error = ();
    fn from_cql(result: &CqlType) -> Result<Self, Self::Error> {
        match result {
            CqlType::Timestamp(timestamp) => {
                time::OffsetDateTime::from_unix_timestamp(*timestamp).map_err(|_| ())
            }
            _ => Err(()),
        }
    }
}

pub trait FromCqlData: Sized {
    type Error;
    fn from_cql(result: &CqlType) -> Result<Self, Self::Error>;
}

impl FromCqlData for String {
    type Error = ();
    fn from_cql(result: &CqlType) -> Result<Self, Self::Error> {
        match result {
            CqlType::Str(s) => Ok(s.clone()),
            _ => Err(()),
        }
    }
}

impl FromCqlData for i64 {
    type Error = ();
    fn from_cql(result: &CqlType) -> Result<Self, Self::Error> {
        match result {
            CqlType::NumInt(num) => Ok(*num),
            _ => Err(()),
        }
    }
}

impl FromCqlData for f64 {
    type Error = ();
    fn from_cql(result: &CqlType) -> Result<Self, Self::Error> {
        match result {
            CqlType::NumFloat(num) => Ok(*num),
            _ => Err(()),
        }
    }
}
// consistency types
pub enum Consistency {
    One,
    Two,
}

pub enum Status {
    Ok,
    NotOk,
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::NotOk => "NotOk",
            Self::Ok => "Ok",
        };
        write!(f, "{}", str)
    }
}
impl ToCqlData for Status {
    fn to_cql(self) -> CqlType {
        CqlType::Str(self.to_string())
    }
}
impl FromCqlData for Status {
    type Error = ();
    fn from_cql(result: &CqlType) -> Result<Self, Self::Error> {
        if let CqlType::Str(s) = result {
            return match s.as_str() {
                "Ok" => Ok(Self::Ok),
                "NotOk" => Ok(Self::NotOk),
                _ => Err(()),
            };
        }
        Err(())
    }
}

pub struct Uuid(pub i64);

impl ToCqlData for Uuid {
    fn to_cql(self) -> CqlType {
        CqlType::NumInt(self.0)
    }
}

impl FromCqlData for Uuid {
    type Error = ();
    fn from_cql(result: &CqlType) -> Result<Self, Self::Error> {
        Ok(Self(i64::from_cql(result)?))
    }
}

// Store Type Definitions
/*
Defining from and to conversions for store types and cql_type
*/
use scylla::deserialize::DeserializeValue;
use scylla::frame::response::result::ColumnType;
use stargate_grpc::{
    proto::{value::Inner, ColumnSpec},
    ResultSet, Row, Value,
};

// astra store type conversions
pub trait IntoValue {
    fn into(self: Box<Self>) -> Value;
}

impl<T> IntoValue for T
where
    T: Into<Value>,
{
    fn into(self: Box<Self>) -> Value {
        Into::into(*self)
    }
}

enum AstraResultIter {
    //curr: Option<CqlMap>,
    Rows(Vec<Row>, Vec<ColumnSpec>),
    Empty,
}

impl AstraResultIter {
    fn new(column: Vec<ColumnSpec>, rows: Vec<Row>) -> Self {
        Self::Rows(rows, column)
    }

    fn empty() -> Self {
        Self::Empty
    }
}

impl Iterator for AstraResultIter {
    type Item = CqlMap;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Rows(from, col_spec) => {
                let curr_row = from.pop()?;
                let zipped = col_spec.clone().into_iter().zip(curr_row.values);

                let mut map: CqlMap = HashMap::with_capacity(zipped.len());
                for (col, value) in zipped {
                    map.insert(col.name.clone(), value.to_cql());
                }
                Some(map)
            }
            Self::Empty => None,
        }
    }
}

impl ToCqlRow for ResultSet {
    type Output = CqlMap;
    fn to_row_iter(self) -> impl Iterator<Item = CqlMap> {
        AstraResultIter::new(self.columns, self.rows)
    }
}

impl ToCqlData for Value {
    fn to_cql(self) -> CqlType {
        if let None = self.inner {
            return CqlType::Null;
        }

        match self.inner.unwrap() {
            Inner::Int(i) => CqlType::NumInt(i),
            Inner::Double(f) => CqlType::NumFloat(f),
            Inner::Float(f) => CqlType::NumFloat(f as f64),
            Inner::Boolean(b) => CqlType::Bool(b),
            Inner::String(s) => CqlType::Str(s),
            Inner::Bytes(b) => CqlType::Bytes(b),
            Inner::Null(_) => CqlType::Null,
            _ => unimplemented!("this type is not implemented"),
        }
    }
}

//TODO : implement for all types
impl Into<Value> for CqlType {
    fn into(self) -> Value {
        match self {
            //CqlType::NumInt(i) => Value::int(i),
            CqlType::Str(s) => Value::string(s),
            _ => unimplemented!("this type is not implemented"),
        }
    }
}

#[derive(Debug)]
struct UnknownType;
impl std::error::Error for UnknownType {}
impl std::fmt::Display for UnknownType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unknown_type")
    }
}

// scylla-cql type conversions
impl<'frame, 'meta> DeserializeValue<'frame, 'meta> for CqlType {
    fn deserialize(
        typ: &'meta scylla::frame::response::result::ColumnType<'meta>,
        v: Option<scylla::deserialize::FrameSlice<'frame>>,
    ) -> Result<Self, scylla::deserialize::DeserializationError> {
        match typ {
            ColumnType::Boolean => Ok(CqlType::Bool(bool::deserialize(typ, v)?)),
            ColumnType::Decimal => Ok(CqlType::NumFloat(f64::deserialize(typ, v)?)),
            ColumnType::Double => Ok(CqlType::NumFloat(f64::deserialize(typ, v)?)),
            ColumnType::Float => Ok(CqlType::NumFloat(f64::deserialize(typ, v)?)),
            ColumnType::Int => Ok(CqlType::NumInt(i64::deserialize(typ, v)?)),
            ColumnType::BigInt => Ok(CqlType::NumInt(i64::deserialize(typ, v)?)),
            ColumnType::Text => Ok(CqlType::Str(String::deserialize(typ, v)?)),
            _other => Err(scylla::deserialize::DeserializationError::new(UnknownType)),
        }
    }

    fn type_check(
        typ: &scylla::frame::response::result::ColumnType,
    ) -> Result<(), scylla::deserialize::TypeCheckError> {
        match typ {
            ColumnType::Boolean
            | ColumnType::Decimal
            | ColumnType::Double
            | ColumnType::Float
            | ColumnType::Int
            | ColumnType::BigInt
            | ColumnType::Text => Ok(()),
            _other => Err(scylla::deserialize::TypeCheckError::new(UnknownType)),
        }
    }
}

impl scylla::serialize::value::SerializeValue for CqlType {
    fn serialize<'b>(
        &self,
        typ: &ColumnType,
        writer: scylla::serialize::writers::CellWriter<'b>,
    ) -> Result<
        scylla::serialize::writers::WrittenCellProof<'b>,
        scylla::serialize::SerializationError,
    > {
        todo!()
    }
}
