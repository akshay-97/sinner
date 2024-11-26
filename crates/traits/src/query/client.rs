use crate::{
    data_types::types::{CqlMap, CqlMapWithQuery, ToCqlRow},
    nosql::interface::NoSql,
    query::query::{Create, FindAll, FindOne, Update},
};
use std::marker::PhantomData;

pub struct FilterBy<T> {
    filter: CqlMap, // TODO: should be impl IntoExpression
    query_string: &'static str,
    _model: PhantomData<fn() -> T>,
}

impl<T> FilterBy<T> {
    pub fn new(filter: CqlMap, query_string: &'static str) -> Self {
        Self {
            filter,
            query_string,
            _model: PhantomData,
        }
    }
}

trait State {}

pub struct Init;
impl State for Init {}

pub struct Ready;
impl State for Ready {}

// for inserts impl by NoSql models
pub trait Insertable: NoSql {
    fn create(self) -> InsertBuilder<Self> {
        InsertBuilder {
            model: self,
            prepared_statement: Some(Self::insert_statement()),
        }
    }
}

pub struct InsertBuilder<T: NoSql> {
    model: T,
    prepared_statement: Option<&'static str>,
    // consistency :  add Consistency
}

impl<T: NoSql> InsertBuilder<T> {
    pub fn build(self) -> Create<T> {
        Create::<T>::create_query(self.model)
    }
}

// for select impl by NoSql models
pub trait Selectable: NoSql {
    fn select() -> SelectBuilder<Self, Init> {
        SelectBuilder::<Self, Init>::default()
    }
    fn select_all() -> SelectAllBuilder<Self, Init> {
        SelectAllBuilder::<Self, Init>::default()
    }
}

pub struct SelectBuilder<T: NoSql, S: State> {
    wh_clause: Option<FilterBy<T>>,
    state: S,
    _model: PhantomData<T>,
}

impl<T: NoSql> SelectBuilder<T, Init> {
    pub fn default() -> Self {
        Self {
            wh_clause: None,
            state: Init,
            _model: PhantomData,
        }
    }

    pub fn filter_by(self, filter: FilterBy<T>) -> SelectBuilder<T, Ready> {
        SelectBuilder {
            wh_clause: Some(filter),
            state: Ready,
            _model: self._model,
        }
    }
}
impl<T: NoSql> SelectBuilder<T, Ready> {
    pub fn build(self) -> FindOne<T> {
        let filter = self.wh_clause.expect("filter not found");
        let query_string = format!(
            "SELECT * FROM {}.{} WHERE {}",
            T::keyspace(),
            T::table_name(),
            filter.query_string
        );
        FindOne::<T>::create_query(filter.filter, query_string)
    }
}

pub struct SelectAllBuilder<T: NoSql, S: State> {
    wh_clause: Option<FilterBy<T>>,
    state: S,
    _model: PhantomData<T>,
}

impl<T: NoSql> SelectAllBuilder<T, Init> {
    pub fn default() -> Self {
        Self {
            wh_clause: None,
            state: Init,
            _model: PhantomData,
        }
    }

    pub fn filter_by(self, filter: FilterBy<T>) -> SelectAllBuilder<T, Ready> {
        SelectAllBuilder {
            wh_clause: Some(filter),
            state: Ready,
            _model: self._model,
        }
    }
}
impl<T: NoSql> SelectAllBuilder<T, Ready> {
    pub fn build(self) -> FindAll<T> {
        let filter = self.wh_clause.expect("filter not found");
        let query_string = format!(
            "SELECT * FROM {}.{} WHERE {}",
            T::keyspace(),
            T::table_name(),
            filter.query_string
        );
        FindAll::<T>::create_query(filter.filter, query_string)
    }
}

// for update impl by NoSql models
pub trait Updateable: ToCqlRow<Output = CqlMapWithQuery> + Sized {
    type ParentModel: NoSql;
    fn update(self) -> UpdateBuilder<Self::ParentModel, Init> {
        let set_clause = self
            .to_row_iter()
            .next()
            .expect("Update struct to cql row conversion failed");
        UpdateBuilder::<Self::ParentModel, Init>::new(set_clause)
    }
}

pub struct UpdateBuilder<T: NoSql, S: State> {
    set_clause: CqlMapWithQuery,
    wh_clause: Option<FilterBy<T>>,
    state: S,
    _model: PhantomData<T>,
}

impl<T: NoSql> UpdateBuilder<T, Init> {
    pub fn new(set_clause: CqlMapWithQuery) -> Self {
        Self {
            set_clause,
            wh_clause: None,
            state: Init,
            _model: PhantomData,
        }
    }

    pub fn filter_by(self, filter: FilterBy<T>) -> UpdateBuilder<T, Ready> {
        UpdateBuilder {
            set_clause: self.set_clause,
            wh_clause: Some(filter),
            state: Ready,
            _model: self._model,
        }
    }
}
impl<T: NoSql> UpdateBuilder<T, Ready> {
    pub fn build(self) -> Update<T> {
        let filter = self.wh_clause.expect("filter not found");
        let query_string = format!(
            "UPDATE {}.{} SET {} WHERE {}",
            T::keyspace(),
            T::table_name(),
            self.set_clause.0,
            filter.query_string
        );
        Update::<T>::create_query(filter.filter, self.set_clause.1, query_string)
    }
}
