mod conds;
pub mod query;
mod select;

pub trait QueryBuilder {
    fn walk_ast(&self, query: &mut query::CassandraQuery);
}

#[cfg(test)]
mod tests {
    use super::select::SelectBuilder;
    use crate::data_types::types::CqlMap;

    #[test]
    fn select_all_with_clause() {
        let select = SelectBuilder::<u32>::new(String::from("Payments"), CqlMap::new())
            .wh()
            .eq(String::from("payment_id"))
            .limit(2);

        let expected = "SELECT * FROM Payments WHERE payment_id = ? LIMIT 2 ";
        assert_eq!(select, expected);
    }

    #[test]
    fn select_all_and_or_clause() {
        let select = SelectBuilder::<u32>::new(String::from("Payments"), CqlMap::new())
            .wh()
            .eq(String::from("payment_id"))
            .and()
            .eq(String::from("status"))
            .or()
            .eq(String::from("currency"))
            .limit(2);

        let expected =
            "SELECT * FROM Payments WHERE payment_id = ? AND status = ? OR currency = ? LIMIT 2 ";
        assert_eq!(select, expected);
    }

    #[test]
    fn select_all_without_clause() {
        let select = SelectBuilder::<u32>::new(String::from("Payments"), CqlMap::new()).build();

        let expected = "SELECT * FROM Payments ";
        assert_eq!(select, expected);
    }
}
