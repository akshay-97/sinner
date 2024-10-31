use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum CqlType{
    Str(String),
    Bool(bool),
    Row(CqlMap),
    NumInt(i64),
    NumFloat(f64),
    Bytes(Vec<u8>),
    Null,
}

pub type CqlMap  = HashMap<String, CqlType>;

pub trait ToCqlData{
    fn to_cql(self) -> CqlType;
}

pub trait ToCqlRow{
   // fn to_row(self) -> CqlMap;
    //fn to_row_iter<T, S : Iterator<Item = T>>(self: dyn IntoIterator<Item = T, IntoIter = S>) -> impl IntoIterator;
    fn to_row_iter(self) -> impl Iterator<Item = CqlMap>;
}   

impl ToCqlData for String{
    fn to_cql(self) -> CqlType {
        CqlType::Str(self.clone())
    }
}

impl ToCqlData for i64{
    fn to_cql(self) -> CqlType {
        CqlType::NumInt(self.clone())
    }
}

impl ToCqlData for f64{
    fn to_cql(self) -> CqlType {
        CqlType::NumFloat(self.clone())
    }
}

pub trait FromCqlData :Sized {
    type Error;
    fn from_cql(result: &CqlType) -> Result<Self , Self::Error>;
}

impl FromCqlData for String{
    type Error = ();
    fn from_cql(result: &CqlType) -> Result<Self , Self::Error> {
        match result{
            CqlType::Str(s) => {
                Ok(s.clone())
            }
            _ => Err(())
        }
    }
}

impl FromCqlData for i64{
    type Error = ();
    fn from_cql(result: &CqlType) -> Result<Self , Self::Error> {
        match result{
            CqlType::NumInt(num) => Ok(*num),
            _ => Err(())
        }
    }
}

impl FromCqlData for f64{
    type Error = ();
    fn from_cql(result: &CqlType) -> Result<Self , Self::Error> {
        match result{
            CqlType::NumFloat(num) => Ok(*num),
            _ => Err(())
        }
    }
}
// consistency types
pub enum Consistency{
    One, Two
}

pub enum Status{
    Ok,
    NotOk
}

impl std::fmt::Display for Status{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self{
            Self::NotOk => "NotOk",
            Self::Ok => "Ok", 
        };
        write!(f, "{}", str)
    }
}
impl ToCqlData for Status{
    fn to_cql(self) -> CqlType {
        CqlType::Str(self.to_string())
    }
}
impl FromCqlData for Status{
    type Error = ();
    fn from_cql(result: &CqlType) -> Result<Self , Self::Error> {
        if let CqlType::Str(s) = result{
            return (match s.as_str(){
                "Ok" => Ok(Self::Ok),
                "NotOk" => Ok(Self::NotOk),
                _ => Err(())
            })
        }
        Err(())
    }
}
pub struct Uuid(i64);

impl ToCqlData for Uuid{
    fn to_cql(self) -> CqlType {
        CqlType::NumInt(self.0)
    }
}

impl FromCqlData for Uuid{
    type Error = ();
    fn from_cql(result: &CqlType) -> Result<Self , Self::Error> {
        Ok(Self(i64::from_cql(result)?))
    }
}

//
use crate::{nosql::interface::CqlStore, query::query::QueryInterface};
use stargate_grpc::{proto::{value::Inner, ColumnSpec}, query::QueryBuilder, Query, ResultSet, Row, Value};

pub trait IntoValue{
    fn into(self: Box<Self>) -> Value;
}

impl <T> IntoValue for T
where 
    T : Into<Value>
{
    fn into(self: Box<Self>) -> Value{
        Into::into(*self)
    }
}


pub struct AstraResult(Vec<ColumnSpec>, Vec<Row>);

pub struct AstrStatement{
    query_str : &'static str,
    binds : Vec<(String, Box<dyn IntoValue + Send>)>,
    keyspace : &'static str,
}

impl AstrStatement{
    pub fn new(query_str : &'static str
        , binds: Vec<(String, Box<dyn IntoValue + Send>)>
        , keyspace : &'static str) -> Self
    {
        Self
            {
                query_str,
                binds,
                keyspace
            }
    }
}

#[async_trait::async_trait]
impl CqlStore for stargate_grpc::StargateClient{
    type Output = AstraResult;
    type Statement = AstrStatement;
    type StoreError = ();
    type Query = QueryBuilder;

    async fn execute(&mut self, statement: Self::Statement) -> Result<Self::Output, Self::StoreError> {
        let query = Self::into_query(statement).build();
        self.execute_query(query)
            .await
            .map_err(|_| ())
            .and_then(|r| {
                let result_set: ResultSet = r.try_into().map_err(|_| ())?;
                Ok(AstraResult(result_set.columns, result_set.rows))
            })
            .map_err(|_e| ())
    }

    fn into_query(statement : Self::Statement) -> Self::Query{
        let mut query = Query::builder()
            .keyspace(statement.keyspace)
            .query(statement.query_str);
        
        let mut enumer = statement.binds
            .into_iter();
        for (el, value) in enumer.next(){
            query = query.bind_name(el.as_str(), value.into());
        }
        query
    }
}

struct AstraResultIter{
    //curr: Option<CqlMap>,
    from : Vec<Row>,
    col_spec : Vec<ColumnSpec>, 
}

impl AstraResultIter{
    fn new(astra_result : AstraResult) -> Self{
        Self{
           // curr : None,
            from : astra_result.1,
            col_spec: astra_result.0,
        }
    }
}

impl Iterator for AstraResultIter{
    type Item = CqlMap;

    fn next(&mut self) -> Option<Self::Item>{
        let curr_row = self.from.pop()?;
        let zipped = self.col_spec.clone()
            .into_iter()
            .zip(curr_row.values);

        let mut map : CqlMap = HashMap::with_capacity(zipped.len());
        for (col, value) in zipped{
            map.insert(col.name.clone(), value.to_cql());
        }
            Some(map)
    }
}
impl ToCqlRow for AstraResult{
    
    fn to_row_iter(self) -> impl Iterator<Item = CqlMap> {
        AstraResultIter::new(self)
    }
}

impl ToCqlData for Value{
    fn to_cql(self) -> CqlType {
        if let None = self.inner{
            return CqlType::Null
        }

        match self.inner.unwrap(){
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

impl Into<Value> for CqlType{
    fn into(self) -> Value {
        match self {
            //CqlType::NumInt(i) => Value::int(i),
            CqlType::Str(s) => Value::string(s),
            _ => unimplemented!("this type is not implemented"),
        }
    }
}