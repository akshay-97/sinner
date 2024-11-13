mod clients;
use anyhow::Context;
use stargate_grpc::*;
use std::iter::Filter;
use std::str::FromStr;
use traits::query::query::{FindOne, Create, QueryInterface, FindAll, Update};
use traits::data_types::types::{CqlMap, ToCqlRow, CqlMapWithQuery};
use std::marker::PhantomData;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = get_astra_client().await.expect("client connect failed");
    let payment = Payment::default();
    //let query_2 : FindOne<Payment> = Payment::find_by_primary_key(10i64);

    let create_query = payment.create().build();
    let find_query =
        Payment::select()
            .filter_by(Payment::filter_by_id(1i64))
            .build();

    let update_payment = UpdatePayment{status : "asdasd".to_string()};
    let update_query =
        update_payment
            .update()
            .filter_by(Payment::filter_by_id(1i64))
            .build();
    let res_1 = create_query.execute(&mut client).await;
    let res_2 = find_query.execute(&mut client).await;
    //let res_3 = update_query.execute(&mut client).await;
    
    Ok(())
}

async fn get_astra_client() -> anyhow::Result<StargateClient>{
    let url: &str  = "asd";
    let token = "token";
    Ok(StargateClient::builder()
        .uri(url)?
        .auth_token(AuthToken::from_str(token)?)
        .connect()
        .await?
    )
}
/*
[ExprAssign { attrs: [], left: Expr::Path { attrs: [], qself: None, path: Path { leading_colon: None, segments: [PathSegment { ident: Ident { ident: "primary_key", span: #0 bytes(89..100) }, arguments: PathArguments::None }] } }, eq_token: Eq, right: Expr::Array { attrs: [], bracket_token: Bracket, elems: [Expr::Lit { attrs: [], lit: Lit::Str { token: "name" } }] } }, ExprAssign { attrs: [], left: Expr::Path { attrs: [], qself: None, path: Path { leading_colon: None, segments: [PathSegment { ident: Ident { ident: "clustering_key", span: #0 bytes(113..127) }, arguments: PathArguments::None }] } }, eq_token: Eq, right: Expr::Array { attrs: [], bracket_token: Bracket, elems: [Expr::Lit { attrs: [], lit: Lit::Str { token: "path" } }, Comma, Expr::Lit { attrs: [], lit: Lit::Str { token: "another" } }] } }, ExprAssign { attrs: [], left: Expr::Path { attrs: [], qself: None, path: Path { leading_colon: None, segments: [PathSegment { ident: Ident { ident: "table", span: #0 bytes(151..156) }, arguments: PathArguments::None }] } }, eq_token: Eq, right: Expr::Lit { attrs: [], lit: Lit::Str { token: "and" } } }, ExprAssign { attrs: [], left: Expr::Path { attrs: [], qself: None, path: Path { leading_colon: None, segments: [PathSegment { ident: Ident { ident: "keyspace", span: #0 bytes(166..174) }, arguments: PathArguments::None }] } }, eq_token: Eq, right: Expr::Lit { attrs: [], lit: Lit::Str { token: "table" } } }]
*/

use std::collections::HashMap;
use traits::{data_types::types::{CqlType, FromCqlData, Status, ToCqlData, Uuid}, nosql::interface::{CqlStore, NoSql}};

#[derive(sin::ToCqlData, sin::FromCqlData)]
struct User{
    user_id : String,
    age : i64,
}

#[derive(Default)]
#[sin::nosql(partition_key = [id], clustering_key = [status], keyspace = test, table = payments)]
struct Payment{
    id : i64,
    status : String,
    amount : f64,
}

struct UpdatePayment{
    status : String
}

impl Updateable for UpdatePayment{
    type ParentModel = Payment;
}

impl ToCqlRow for UpdatePayment{
    type Output = CqlMapWithQuery;

    fn to_row_iter(self) -> impl Iterator<Item = Self::Output>{
        let set_clause_q = "status = ?";
        let set_clause = HashMap::from([
            ("status".to_string(), self.status.to_cql())
        ]);
        std::iter::once((set_clause_q.to_string(), set_clause))
    }
}

struct FilterBy<T>{
    filter : CqlMap, // TODO: should be impl IntoExpression
    query_string : &'static str,
    _model : PhantomData<fn() -> T>,
}

impl <T> FilterBy<T>{
    pub fn new(filter : CqlMap, query_string: &'static str) -> Self{
        Self{
            filter,
            query_string,
            _model : PhantomData
        }
    }
}


pub trait Insertable : NoSql{
    fn create(self) -> InsertBuilder<Self>{
        InsertBuilder{
            model : self,
            prepared_statement : Some(Self::insert_statement()),
        }
    }
}

pub struct InsertBuilder<T: NoSql>{
    model : T,
    prepared_statement : Option<&'static str>,
    // consistency :  add Consistency
}

impl <T: NoSql> InsertBuilder<T>{
    pub fn build(self) -> Create<T>{
        Create::<T>::create_query(self.model)
    }
}
pub trait Selectable : NoSql{
    fn select() -> SelectBuilder<Self, Init>{
        SelectBuilder::<Self, Init>::default()
    }
    fn select_all() -> SelectAllBuilder<Self, Init>{
        SelectAllBuilder::<Self, Init>::default()
    }
}

trait State {}

struct Init;
impl State for Init{}
// struct Filtered;
// impl State for Filtered{}
struct Ready;
impl State for Ready{}
pub struct SelectBuilder<T : NoSql, S : State>{
    wh_clause : Option<FilterBy<T>>,
    state : S,
    _model : PhantomData<T>
}

impl <T: NoSql> SelectBuilder<T, Init>{
    pub fn default() -> Self{
        Self{
            wh_clause : None,
            state : Init,
            _model : PhantomData
        }
    }

    pub fn filter_by(self , filter : FilterBy<T>) -> SelectBuilder<T, Ready>{ 
        SelectBuilder{
            wh_clause : Some(filter),
            state : Ready,
            _model : self._model,
        }
    }
}
impl <T: NoSql> SelectBuilder<T, Ready>{
    pub fn build(self) -> FindOne<T>{
        let filter = self.wh_clause.expect("filter not found");
        let query_string = format!("SELECT * FROM {}.{} WHERE {}", T::keyspace(), T::table_name(), filter.query_string);
        FindOne::<T>::create_query(filter.filter, query_string)
    }
}

pub struct SelectAllBuilder<T : NoSql, S: State>{
    wh_clause : Option<FilterBy<T>>,
    state : S,
    _model : PhantomData<T>
}

impl <T: NoSql> SelectAllBuilder<T, Init>{
    pub fn default() -> Self{
        Self{
            wh_clause : None,
            state : Init,
            _model : PhantomData
        }
    }

    pub fn filter_by(self , filter : FilterBy<T>) -> SelectAllBuilder<T, Ready>{ 
        SelectAllBuilder{
            wh_clause : Some(filter),
            state : Ready,
            _model : self._model,
        }
    }
}
impl <T: NoSql> SelectAllBuilder<T, Ready>{
    pub fn build(self) -> FindAll<T>{
        let filter = self.wh_clause.expect("filter not found");
        let query_string = format!("SELECT * FROM {}.{} WHERE {}", T::keyspace(), T::table_name(), filter.query_string);
        FindAll::<T>::create_query(filter.filter, query_string)
    }
}

pub trait Updateable : ToCqlRow<Output = CqlMapWithQuery> + Sized{
    type ParentModel: NoSql;
    fn update(self) -> UpdateBuilder<Self::ParentModel, Init>{
        let set_clause = self.to_row_iter().next().expect("Update struct to cql row conversion failed");
        UpdateBuilder::<Self::ParentModel, Init>::new(set_clause)
    }
}

pub struct UpdateBuilder<T : NoSql, S: State>{
    set_clause : CqlMapWithQuery,
    wh_clause: Option<FilterBy<T>>,
    state : S,
    _model : PhantomData<T>
}

impl <T: NoSql> UpdateBuilder<T, Init>{
    pub fn new(set_clause : CqlMapWithQuery) -> Self{
        Self{
            set_clause,
            wh_clause : None,
            state: Init,
            _model : PhantomData,
        }
    }

    pub fn filter_by(self , filter : FilterBy<T>) -> UpdateBuilder<T, Ready>{ 
        UpdateBuilder{
            set_clause : self.set_clause,
            wh_clause : Some(filter),
            state : Ready,
            _model : self._model,
        }
    }
}
impl <T: NoSql> UpdateBuilder<T, Ready>{
    pub fn build(self) -> Update<T>{
        let filter = self.wh_clause.expect("filter not found");
        let query_string = format!("UPDATE {}.{} SET {} WHERE {}", T::keyspace(), T::table_name(), self.set_clause.0, filter.query_string);
        Update::<T>::create_query(filter.filter, self.set_clause.1 , query_string)
    }
}
// struct QueryBuilder<T>{
//     state : QueryState,
//     query : impl ToRawStatement,
// }

// enum QueryState{
//     Init,
//     Filter,
//     Build
// }

// impl <T: NoSql> QueryBuilder<T>{
//     pub fn create(new: T) -> Self{
//         Self{
//             state : QueryState::Create(new)
//         }
//     }

//     pub fn find() -> Self{
//         Self{

//         }
//     }
// }

// struct CreateQ<T>(T);

// impl <T> CreateQ<T>{
//     fn init(model : T) -> Self{
//         Self(model)
//     }
// }

// impl <T> Into<Create<T>> for CreateState<T>
//     where T: NoSql
// {
//     fn into(self) -> Create<T> {
//         Create::<T>::create_query(self.0)
//     }
// }
// struct UpdateQ<T>(CqlMap, Option<FilterBy<T>>);



// struct DeleteQ<T>(Option<FilterBy<T>>);

// struct FindQ<T>(Option<FilterBy<T>>);

// enum QueryType<T>{
//     Create(CreateQ<T>),
//     Update(UpdateQ<T>),
//     Delete(DeleteQ<T>),
//     Find(FindQ<T>)
// }

// impl <T: NoSql> FilterBy<T>{
//     fn new(filter : CqlMap) -> Self{
//         Self{
//             filter,
//             _model : PhantomData
//         }
//     }
// }




/*
let query  : Query = table
    .create()
    .consistency();

let result = query.run(store).await?;

let query = Table::select()
                .filter(Table::PartitionKey // Table::SaIKey)
                .filter(Table::PrimaryCluster)
                .filter(Table::SecondaryCluster)
                .filter(Table::TertiaryCluster)
                .order_by(ASC // DESC)
                .limit(usize)
                .consistency();

let result = query.run(store).await?;

#[table=Table]
// validation that no struct fields contain partition or clustering key
struct UpdateInternal{}
let query = updateInternal.update()
                .filter(Table::PartitionKey//Table::SaIKey)
                .filter(Table::PrimaryCluster)
                .filter(Table::SecondaryCluster)
                .filter(Table::TertiaryCluster)
                .consistency();

struct Query<S: QueryState, T>{
    expr : Expr,
    state : S,
    _model : PhantomData<fn() -> T>
}

struct Open{};
struct Insert{
    prepared_statement : &'static str,
    consistenct : Option<Consistency>,
};
struct Find{
    filter_expr : Option<Expr>
};
struct Update{
    set_data : impl Into<CqlMap>,
    filter_expr : Option<Expr>
};



*/