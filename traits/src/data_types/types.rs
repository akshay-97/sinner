use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum CqlType{
    Str(String),
    Bool(bool),
    Row(CqlMap),
    NumInt(i32),
    NumFloat(f64),
    Null,
}

pub type CqlMap  = HashMap<String, CqlType>;

pub trait ToCqlData{
    fn to_cql(self) -> CqlType;
}

pub trait ToCqlRow{
    fn to_row(self) -> CqlMap;
}

impl ToCqlData for String{
    fn to_cql(self) -> CqlType {
        CqlType::Str(self.clone())
    }
}

impl ToCqlData for i32{
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

impl FromCqlData for i32{
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
pub struct Uuid(i32);

impl ToCqlData for Uuid{
    fn to_cql(self) -> CqlType {
        CqlType::NumInt(self.0)
    }
}

impl FromCqlData for Uuid{
    type Error = ();
    fn from_cql(result: &CqlType) -> Result<Self , Self::Error> {
        Ok(Self(i32::from_cql(result)?))
    }
}