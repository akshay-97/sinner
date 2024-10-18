#[derive(Debug)]
pub enum CqlType{
    Str(String),
    Bool(bool),
    Row(CqlMap),
    NumInt(i32),
    NumFloat(f64),
    Null,
}

type CqlMap  = Vec<(String, CqlType)>;

pub trait ToCqlData{
    fn to_cql(self) -> CqlType;
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


pub trait NoSql : FromCqlData + ToCqlData {
    // fn partition_key() -> [&'static str,usize];
    // fn clustering_key() ->[&'static str, usize];
    fn table_name() -> &'static str;
    fn keyspace() -> &'static str;
}

pub trait FromCqlData :Sized{
    type Error;
    fn from_cql(result: CqlType) -> Result<Self , Self::Error>;
}


///// Query types
/// 
enum Consistency{
    One, Two
}
struct Query<T : NoSql>{
    consistency : Option<Consistency>,
    op : Op,
    _entity: PhantomData<T>
}

enum Op {
    Insert(InsertBody),
    Update(UpdateBody),
    Delete(DeleteBody),
    Find(FindBody),
}

struct InsertBody{
    data : CqlMap
}

struct UpdateBody{
    where_clause : CqlMap,
    set_clause: CqlMap,
}

struct DeleteBody{
    where_clause: CqlMap
}

struct FindBody {
    where_clause : CqlMap,
}


//// StorageInterface
/// 
trait StorageInterface{
    fn execute<T: NoSql>(&self, query: Query<T>) -> QueryResult;

}


type QueryResult = Result<Qres, CqlError>;

enum Qres{
    Success(Vec<u8>),
    NoRes(),
    SuccessCount(usize),
}

enum CqlError{
    E01,
    E02,
    E03
}
