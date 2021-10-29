/// Macro rule to execute database query
///
/// __connection__ : database connection <br>
/// __query__ : query to execute <br>
/// __params__ : query params
#[macro_export]
macro_rules! execute_query {
    (pool <- $pool:expr, query <- $query: expr, params <- $params: expr) => {{
        let db_pool: &DBPool = $pool;
        let db_conn = get_connection(db_pool).await?;
        let q: String = $query;
        let p: &[&(dyn Sync + ToSql)] = $params;

        db_conn
            .query_one(q.as_str(), p)
            .await
            .map_err(|source| CustomError::ExecuteDBQueryError { source, query: q })?
    }};
}

/// Macro rule to execute database query that returns an Option<Row>
///
/// __connection__ : database connection <br>
/// __query__ : query to execute <br>
/// __params__ : query params
#[macro_export]
macro_rules! execute_query_opt {
    (pool <- $pool:expr, query <- $query: expr, params <- $params: expr) => {{
        let db_pool: &DBPool = $pool;
        let db_conn = get_connection(db_pool).await?;
        let q: String = $query;
        let p: &[&(dyn Sync + ToSql)] = $params;
        db_conn
            .query_opt(q.as_str(), p)
            .await
            .map_err(|source| CustomError::ExecuteDBQueryError { source, query: q })?
    }};
}
