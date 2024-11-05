mod clients;
use stargate_grpc::*;
use std::str::FromStr;
use traits::query::query::{FindOne, Create, QueryInterface};
use traits::data_types::types::CqlMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = get_astra_client().await.expect("client connect failed");
    let payment = Payment::create_payment(100f64);

    let query_1 : Create<Payment> = Create::create_query(payment);
    let query_2 : FindOne<Payment> = Payment::find_by_primary_key(10i64);

    let res_1 = query_1.execute(&mut client).await;
    let res_2 = query_2.execute(&mut client).await;
    
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


#[sin::nosql(partition_key = [id], clustering_key = [status], keyspace = test, table = payments)]
struct Payment{
    id : Uuid,
    status : Status,
    amount : f64,
}

impl Payment{
    fn create_payment(amount : f64) -> Self{
        Self{
            id : Uuid(0i64),
            status  : Status::NotOk,
            amount
        }
    }

    fn find_by_primary_key(id : i64) -> FindOne<Payment>{
        let binds : CqlMap = HashMap::from([("id".to_string(), id.to_cql())]);
        let query = "SELECT * FROM test.payments where id = ?".to_string();
        FindOne::<Payment>::create_query(binds, query)
    }
}