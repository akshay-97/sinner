fn main() {
    let user = User { user_id : "asdb".to_string(), age : 123};
    println!("user cql is {:?} ,", user.to_cql());

    user.find(where_clause);

}

/*

[ExprAssign { attrs: [], left: Expr::Path { attrs: [], qself: None, path: Path { leading_colon: None, segments: [PathSegment { ident: Ident { ident: "primary_key", span: #0 bytes(89..100) }, arguments: PathArguments::None }] } }, eq_token: Eq, right: Expr::Array { attrs: [], bracket_token: Bracket, elems: [Expr::Lit { attrs: [], lit: Lit::Str { token: "name" } }] } }, ExprAssign { attrs: [], left: Expr::Path { attrs: [], qself: None, path: Path { leading_colon: None, segments: [PathSegment { ident: Ident { ident: "clustering_key", span: #0 bytes(113..127) }, arguments: PathArguments::None }] } }, eq_token: Eq, right: Expr::Array { attrs: [], bracket_token: Bracket, elems: [Expr::Lit { attrs: [], lit: Lit::Str { token: "path" } }, Comma, Expr::Lit { attrs: [], lit: Lit::Str { token: "another" } }] } }, ExprAssign { attrs: [], left: Expr::Path { attrs: [], qself: None, path: Path { leading_colon: None, segments: [PathSegment { ident: Ident { ident: "table", span: #0 bytes(151..156) }, arguments: PathArguments::None }] } }, eq_token: Eq, right: Expr::Lit { attrs: [], lit: Lit::Str { token: "and" } } }, ExprAssign { attrs: [], left: Expr::Path { attrs: [], qself: None, path: Path { leading_colon: None, segments: [PathSegment { ident: Ident { ident: "keyspace", span: #0 bytes(166..174) }, arguments: PathArguments::None }] } }, eq_token: Eq, right: Expr::Lit { attrs: [], lit: Lit::Str { token: "table" } } }]
*/

use std::{collections::HashMap};

use traits::{data_types::types::{CqlType, FromCqlData, ToCqlData, Status, Uuid}, nosql::interface::{NoSql, StorageInterface}
    , query::query::*};

#[derive(sin::ToCqlData, sin::FromCqlData)]
struct User{
    user_id : String,
    age : i32,
}


#[sin::nosql(partition_key = [id], clustering_key = [status], keyspace = test, table = payments)]
struct Payment{
    id : Uuid,
    status : Status,
    amount : f64,
}


