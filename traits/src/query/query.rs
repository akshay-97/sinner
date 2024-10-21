use crate::data_types::types::{CqlMap, Consistency};
use crate::nosql::interface::NoSql;
use std::marker::PhantomData;

pub struct Query<T : NoSql>{
    consistency : Option<Consistency>,
    op : Op,
    _entity: PhantomData<T>
}

impl <T : NoSql> Query<T>{
    pub fn create_find_query(query_clause : CqlMap, c : Option<Consistency>) -> Self{
        Self{
            consistency : c,
            op : Op::Find(FindBody::new(query_clause)),
            _entity : PhantomData,
        }
    }
}

pub enum Op {
    Insert(InsertBody),
    Update(UpdateBody),
    Delete(DeleteBody),
    Find(FindBody),
}

pub struct InsertBody{
    data : CqlMap
}

pub struct UpdateBody{
    where_clause : CqlMap,
    set_clause: CqlMap,
}

pub struct DeleteBody{
    where_clause: CqlMap
}

pub struct FindBody {
    where_clause : CqlMap,
}

impl FindBody{
    pub fn new(where_clause : CqlMap) -> Self{
        Self{
            where_clause
        }
    }
}
