// This file was generated with `cornucopia`. Do not modify.

#[derive(Clone, Copy, Debug)]
pub struct ActivityForAliasParams {
    pub alias_id: i64,
    pub lim: i64,
    pub off: i64,
}
#[derive(Clone, Copy, Debug)]
pub struct ActivityForAliasApiParams {
    pub alias_id: i64,
    pub lim: i64,
    pub off: i64,
}
#[derive(Debug)]
pub struct InsertBlockParams<T1: crate::StringSql> {
    pub alias_id: i64,
    pub from_address: Option<T1>,
}
#[derive(Debug)]
pub struct InsertForwardParams<T1: crate::StringSql> {
    pub alias_id: i64,
    pub from_address: Option<T1>,
}
#[derive(Debug)]
pub struct FlipFailedParams<T1: crate::StringSql> {
    pub reason: Option<T1>,
    pub email_log_id: i64,
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ActivityForAlias {
    pub action: String,
    pub from_address: Option<String>,
    pub reason: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: time::OffsetDateTime,
}
pub struct ActivityForAliasBorrowed<'a> {
    pub action: &'a str,
    pub from_address: Option<&'a str>,
    pub reason: Option<&'a str>,
    pub created_at: time::OffsetDateTime,
}
impl<'a> From<ActivityForAliasBorrowed<'a>> for ActivityForAlias {
    fn from(
        ActivityForAliasBorrowed {
            action,
            from_address,
            reason,
            created_at,
        }: ActivityForAliasBorrowed<'a>,
    ) -> Self {
        Self {
            action: action.into(),
            from_address: from_address.map(|v| v.into()),
            reason: reason.map(|v| v.into()),
            created_at,
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ActivityForAliasApi {
    pub id: i64,
    pub alias_id: i64,
    pub reverse_contact_id: Option<i64>,
    pub action: String,
    pub from_address: Option<String>,
    pub message_id: Option<String>,
    pub reason: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: time::OffsetDateTime,
}
pub struct ActivityForAliasApiBorrowed<'a> {
    pub id: i64,
    pub alias_id: i64,
    pub reverse_contact_id: Option<i64>,
    pub action: &'a str,
    pub from_address: Option<&'a str>,
    pub message_id: Option<&'a str>,
    pub reason: Option<&'a str>,
    pub created_at: time::OffsetDateTime,
}
impl<'a> From<ActivityForAliasApiBorrowed<'a>> for ActivityForAliasApi {
    fn from(
        ActivityForAliasApiBorrowed {
            id,
            alias_id,
            reverse_contact_id,
            action,
            from_address,
            message_id,
            reason,
            created_at,
        }: ActivityForAliasApiBorrowed<'a>,
    ) -> Self {
        Self {
            id,
            alias_id,
            reverse_contact_id,
            action: action.into(),
            from_address: from_address.map(|v| v.into()),
            message_id: message_id.map(|v| v.into()),
            reason: reason.map(|v| v.into()),
            created_at,
        }
    }
}
use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
pub struct ActivityForAliasQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<ActivityForAliasBorrowed, tokio_postgres::Error>,
    mapper: fn(ActivityForAliasBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> ActivityForAliasQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(ActivityForAliasBorrowed) -> R,
    ) -> ActivityForAliasQuery<'c, 'a, 's, C, R, N> {
        ActivityForAliasQuery {
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
pub struct ActivityForAliasApiQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor:
        fn(&tokio_postgres::Row) -> Result<ActivityForAliasApiBorrowed, tokio_postgres::Error>,
    mapper: fn(ActivityForAliasApiBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> ActivityForAliasApiQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(ActivityForAliasApiBorrowed) -> R,
    ) -> ActivityForAliasApiQuery<'c, 'a, 's, C, R, N> {
        ActivityForAliasApiQuery {
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
pub struct ActivityForAliasStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn activity_for_alias() -> ActivityForAliasStmt {
    ActivityForAliasStmt(
        "SELECT action, from_address, reason, created_at FROM email_log WHERE alias_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        None,
    )
}
impl ActivityForAliasStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient>(
        &'s self,
        client: &'c C,
        alias_id: &'a i64,
        lim: &'a i64,
        off: &'a i64,
    ) -> ActivityForAliasQuery<'c, 'a, 's, C, ActivityForAlias, 3> {
        ActivityForAliasQuery {
            client,
            params: [alias_id, lim, off],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<ActivityForAliasBorrowed, tokio_postgres::Error> {
                Ok(ActivityForAliasBorrowed {
                    action: row.try_get(0)?,
                    from_address: row.try_get(1)?,
                    reason: row.try_get(2)?,
                    created_at: row.try_get(3)?,
                })
            },
            mapper: |it| ActivityForAlias::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        ActivityForAliasParams,
        ActivityForAliasQuery<'c, 'a, 's, C, ActivityForAlias, 3>,
        C,
    > for ActivityForAliasStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a ActivityForAliasParams,
    ) -> ActivityForAliasQuery<'c, 'a, 's, C, ActivityForAlias, 3> {
        self.bind(client, &params.alias_id, &params.lim, &params.off)
    }
}
pub struct ActivityForAliasApiStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn activity_for_alias_api() -> ActivityForAliasApiStmt {
    ActivityForAliasApiStmt(
        "SELECT id, alias_id, reverse_contact_id, action, from_address, message_id, reason, created_at FROM email_log WHERE alias_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        None,
    )
}
impl ActivityForAliasApiStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient>(
        &'s self,
        client: &'c C,
        alias_id: &'a i64,
        lim: &'a i64,
        off: &'a i64,
    ) -> ActivityForAliasApiQuery<'c, 'a, 's, C, ActivityForAliasApi, 3> {
        ActivityForAliasApiQuery {
            client,
            params: [alias_id, lim, off],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<ActivityForAliasApiBorrowed, tokio_postgres::Error> {
                Ok(ActivityForAliasApiBorrowed {
                    id: row.try_get(0)?,
                    alias_id: row.try_get(1)?,
                    reverse_contact_id: row.try_get(2)?,
                    action: row.try_get(3)?,
                    from_address: row.try_get(4)?,
                    message_id: row.try_get(5)?,
                    reason: row.try_get(6)?,
                    created_at: row.try_get(7)?,
                })
            },
            mapper: |it| ActivityForAliasApi::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        ActivityForAliasApiParams,
        ActivityForAliasApiQuery<'c, 'a, 's, C, ActivityForAliasApi, 3>,
        C,
    > for ActivityForAliasApiStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a ActivityForAliasApiParams,
    ) -> ActivityForAliasApiQuery<'c, 'a, 's, C, ActivityForAliasApi, 3> {
        self.bind(client, &params.alias_id, &params.lim, &params.off)
    }
}
pub struct InsertBlockStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn insert_block() -> InsertBlockStmt {
    InsertBlockStmt(
        "INSERT INTO email_log (alias_id, action, status, from_address) VALUES ($1, 'block', 'submitted', $2)",
        None,
    )
}
impl InsertBlockStmt {
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
        from_address: &'a Option<T1>,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[alias_id, from_address]).await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::StringSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        InsertBlockParams<T1>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for InsertBlockStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a InsertBlockParams<T1>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.alias_id, &params.from_address))
    }
}
pub struct InsertForwardStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn insert_forward() -> InsertForwardStmt {
    InsertForwardStmt(
        "INSERT INTO email_log (alias_id, action, from_address) VALUES ($1, 'forward', $2) RETURNING id",
        None,
    )
}
impl InsertForwardStmt {
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
        alias_id: &'a i64,
        from_address: &'a Option<T1>,
    ) -> I64Query<'c, 'a, 's, C, i64, 2> {
        I64Query {
            client,
            params: [alias_id, from_address],
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
        InsertForwardParams<T1>,
        I64Query<'c, 'a, 's, C, i64, 2>,
        C,
    > for InsertForwardStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a InsertForwardParams<T1>,
    ) -> I64Query<'c, 'a, 's, C, i64, 2> {
        self.bind(client, &params.alias_id, &params.from_address)
    }
}
pub struct FlipFailedStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn flip_failed() -> FlipFailedStmt {
    FlipFailedStmt(
        "UPDATE email_log SET status = 'failed', reason = $1 WHERE id = $2 AND status = 'pending'",
        None,
    )
}
impl FlipFailedStmt {
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
        reason: &'a Option<T1>,
        email_log_id: &'a i64,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[reason, email_log_id]).await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::StringSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        FlipFailedParams<T1>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for FlipFailedStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a FlipFailedParams<T1>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.reason, &params.email_log_id))
    }
}
pub struct FlipSubmittedStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn flip_submitted() -> FlipSubmittedStmt {
    FlipSubmittedStmt(
        "UPDATE email_log SET status = 'submitted' WHERE id = $1 AND status = 'pending'",
        None,
    )
}
impl FlipSubmittedStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient>(
        &'s self,
        client: &'c C,
        email_log_id: &'a i64,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[email_log_id]).await
    }
}
