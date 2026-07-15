// This file was generated with `cornucopia`. Do not modify.

#[derive(Debug)]
pub struct CheckParams<T1: crate::StringSql> {
    pub key: T1,
    pub now: time::OffsetDateTime,
    pub window_start_min: time::OffsetDateTime,
}
use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
pub struct I32Query<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<i32, tokio_postgres::Error>,
    mapper: fn(i32) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> I32Query<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(self, mapper: fn(i32) -> R) -> I32Query<'c, 'a, 's, C, R, N> {
        I32Query {
            client: self.client,
            params: self.params,
            query: self.query,
            cached: self.cached,
            extractor: self.extractor,
            mapper,
        }
    }
    pub async fn one(self) -> Result<T, tokio_postgres::Error> {
        let row =
            crate::client::async_::one(self.client, self.query, &self.params, self.cached).await?;
        Ok((self.mapper)((self.extractor)(&row)?))
    }
    pub async fn all(self) -> Result<Vec<T>, tokio_postgres::Error> {
        self.iter().await?.try_collect().await
    }
    pub async fn opt(self) -> Result<Option<T>, tokio_postgres::Error> {
        let opt_row =
            crate::client::async_::opt(self.client, self.query, &self.params, self.cached).await?;
        Ok(opt_row
            .map(|row| {
                let extracted = (self.extractor)(&row)?;
                Ok((self.mapper)(extracted))
            })
            .transpose()?)
    }
    pub async fn iter(
        self,
    ) -> Result<
        impl futures::Stream<Item = Result<T, tokio_postgres::Error>> + 'c,
        tokio_postgres::Error,
    > {
        let stream = crate::client::async_::raw(
            self.client,
            self.query,
            crate::slice_iter(&self.params),
            self.cached,
        )
        .await?;
        let mapped = stream
            .map(move |res| {
                res.and_then(|row| {
                    let extracted = (self.extractor)(&row)?;
                    Ok((self.mapper)(extracted))
                })
            })
            .into_stream();
        Ok(mapped)
    }
}
pub struct CheckStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn check() -> CheckStmt {
    CheckStmt(
        "INSERT INTO rate_limit_bucket (key, count, window_start) VALUES ($1, 1, $2) ON CONFLICT (key) DO UPDATE SET count = CASE WHEN rate_limit_bucket.window_start < $3 THEN 1 ELSE rate_limit_bucket.count + 1 END, window_start = CASE WHEN rate_limit_bucket.window_start < $3 THEN $2 ELSE rate_limit_bucket.window_start END RETURNING count",
        None,
    )
}
impl CheckStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>(
        &'s self,
        client: &'c C,
        key: &'a T1,
        now: &'a time::OffsetDateTime,
        window_start_min: &'a time::OffsetDateTime,
    ) -> I32Query<'c, 'a, 's, C, i32, 3> {
        I32Query {
            client,
            params: [key, now, window_start_min],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it,
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>
    crate::client::async_::Params<'c, 'a, 's, CheckParams<T1>, I32Query<'c, 'a, 's, C, i32, 3>, C>
    for CheckStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a CheckParams<T1>,
    ) -> I32Query<'c, 'a, 's, C, i32, 3> {
        self.bind(client, &params.key, &params.now, &params.window_start_min)
    }
}
pub struct ClearStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn clear() -> ClearStmt {
    ClearStmt("DELETE FROM rate_limit_bucket WHERE key = $1", None)
}
impl ClearStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>(
        &'s self,
        client: &'c C,
        key: &'a T1,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[key]).await
    }
}
