use stargate_grpc::*;
use std::collections::HashMap;
use std::str::FromStr;
use traits::query::client::*;
use traits::query::query::QueryInterface;
use traits::{
    data_types::types::{CqlMapWithQuery, CqlType, FromCqlData, ToCqlData, ToCqlRow},
    nosql::interface::NoSql,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // create an astra store
    let mut client = get_astra_client().await.expect("client connect failed");

    let payment = Payment::default();

    let create_query = payment.create().build();

    let find_query = Payment::select()
        .filter_by(Payment::filter_by_id(1i64))
        .build();

    let update_payment = UpdatePayment {
        status: "asdasd".to_string(),
    };
    let _update_query = update_payment
        .update()
        .filter_by(Payment::filter_by_id(1i64))
        .build();
    let _res_1 = create_query.execute(&mut client).await;
    let _res_2 = find_query.execute(&mut client).await;
    //let res_3 = update_query.execute(&mut client).await;

    Ok(())
}

async fn get_astra_client() -> anyhow::Result<StargateClient> {
    let url: &str = "asd";
    let token = "token";
    Ok(StargateClient::builder()
        .uri(url)?
        .auth_token(AuthToken::from_str(token)?)
        .connect()
        .await?)
}

#[derive(sin::ToCqlData, sin::FromCqlData)]
struct User {
    user_id: String,
    age: i64,
}

#[derive(Default)]
#[sin::nosql(partition_key = [id], clustering_key = [status], keyspace = test, table = payments)]
struct Payment {
    id: i64,
    status: String,
    amount: f64,
}

struct UpdatePayment {
    status: String,
}

impl Updateable for UpdatePayment {
    type ParentModel = Payment;
}

impl ToCqlRow for UpdatePayment {
    type Output = CqlMapWithQuery;

    fn to_row_iter(self) -> impl Iterator<Item = Self::Output> {
        let set_clause_q = "status = ?";
        let set_clause = HashMap::from([("status".to_string(), self.status.to_cql())]);
        std::iter::once((set_clause_q.to_string(), set_clause))
    }
}
