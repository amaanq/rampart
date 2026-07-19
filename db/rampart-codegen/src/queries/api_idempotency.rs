// This file was generated with `cornucopia`. Do not modify.

#[derive(Debug)]
pub struct ClaimParams<T1: crate::StringSql> {
    pub api_key_id: i64,
    pub idempotency_key: T1,
}
#[derive(Debug)]
pub struct AliasIdParams<T1: crate::StringSql> {
    pub api_key_id: i64,
    pub idempotency_key: T1,
}
#[derive(Debug)]
pub struct FinishParams<T1: crate::StringSql> {
    pub alias_id: i64,
    pub api_key_id: i64,
    pub idempotency_key: T1,
}
use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
pub struct I64Query<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<i64, tokio_postgres::Error>,
    mapper: fn(i64) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> I64Query<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(self, mapper: fn(i64) -> R) -> I64Query<'c, 'a, 's, C, R, N> {
        I64Query {
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
pub struct Optioni64Query<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<Option<i64>, tokio_postgres::Error>,
    mapper: fn(Option<i64>) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> Optioni64Query<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(self, mapper: fn(Option<i64>) -> R) -> Optioni64Query<'c, 'a, 's, C, R, N> {
        Optioni64Query {
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
pub struct ClaimStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn claim() -> ClaimStmt {
    ClaimStmt(
        "INSERT INTO api_idempotency (api_key_id, idempotency_key) VALUES ($1, $2) ON CONFLICT DO NOTHING RETURNING api_key_id",
        None,
    )
}
impl ClaimStmt {
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
        api_key_id: &'a i64,
        idempotency_key: &'a T1,
    ) -> I64Query<'c, 'a, 's, C, i64, 2> {
        I64Query {
            client,
            params: [api_key_id, idempotency_key],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it,
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>
    crate::client::async_::Params<'c, 'a, 's, ClaimParams<T1>, I64Query<'c, 'a, 's, C, i64, 2>, C>
    for ClaimStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a ClaimParams<T1>,
    ) -> I64Query<'c, 'a, 's, C, i64, 2> {
        self.bind(client, &params.api_key_id, &params.idempotency_key)
    }
}
pub struct AliasIdStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn alias_id() -> AliasIdStmt {
    AliasIdStmt(
        "SELECT alias_id FROM api_idempotency WHERE api_key_id = $1 AND idempotency_key = $2",
        None,
    )
}
impl AliasIdStmt {
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
        api_key_id: &'a i64,
        idempotency_key: &'a T1,
    ) -> Optioni64Query<'c, 'a, 's, C, Option<i64>, 2> {
        Optioni64Query {
            client,
            params: [api_key_id, idempotency_key],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it,
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        AliasIdParams<T1>,
        Optioni64Query<'c, 'a, 's, C, Option<i64>, 2>,
        C,
    > for AliasIdStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a AliasIdParams<T1>,
    ) -> Optioni64Query<'c, 'a, 's, C, Option<i64>, 2> {
        self.bind(client, &params.api_key_id, &params.idempotency_key)
    }
}
pub struct FinishStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn finish() -> FinishStmt {
    FinishStmt(
        "UPDATE api_idempotency SET alias_id = $1 WHERE api_key_id = $2 AND idempotency_key = $3",
        None,
    )
}
impl FinishStmt {
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
        alias_id: &'a i64,
        api_key_id: &'a i64,
        idempotency_key: &'a T1,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(self.0, &[alias_id, api_key_id, idempotency_key])
            .await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::StringSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        FinishParams<T1>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for FinishStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a FinishParams<T1>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.alias_id,
            &params.api_key_id,
            &params.idempotency_key,
        ))
    }
}
