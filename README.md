# Rust ORM for Cassandra/Scylla driver and Astra Stargate client

⚠️ This project is currently in an early stage of development. Feedback and contributions are
welcomed!

## Feature overview:
- Provide simple interface to define model using macros
- Auto generate query filters based on table specs
- Support Updates as typed structs
- Supporting scylla-cql and stargate-grpc clients

### Define Tables
see code reference in try_sin crate
  ```rust
#[derive(Default)]
#[sin::nosql(partition_key = [id], clustering_key = [status],
    keyspace = test, table = payments)]
struct Payment{
    id : i64,
    status : String,
    amount : f64,
}
  ```

### Queries Usage
```rust
let create_payment : Create<Payment> = Payment::default()
    .create()
    .build();

/* let client = get_astra_client() -- create underlying driver client that you use for scylla/cassandra. check traits/clients to see supported drivers
*/
create_payment.execute(&mut client).await;

let find_payment = Payment::select()
    .filter_by(Payment::filter_by_id(_))
    .build();

let res = find_payment.execute(&mut client).await;
```
