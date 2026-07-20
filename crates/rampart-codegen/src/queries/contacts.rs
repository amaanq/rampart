// This file was generated with `cornucopia`. Do not modify.

#[derive(Clone, Copy, Debug)]
pub struct ExistsForUserParams {
    pub contact_id: i64,
    pub user_id: i64,
}
#[derive(Clone, Copy, Debug)]
pub struct SetEnabledParams {
    pub enabled: bool,
    pub contact_id: i64,
}
#[derive(Clone, Copy, Debug)]
pub struct SetBlockReplyParams {
    pub block_reply: bool,
    pub contact_id: i64,
}
#[derive(Debug)]
pub struct SetDisplayNameParams<T1: crate::StringSql> {
    pub display_name: Option<T1>,
    pub contact_id: i64,
}
#[derive(Clone, Copy, Debug)]
pub struct DeleteForUserParams {
    pub contact_id: i64,
    pub user_id: i64,
}
#[derive(Debug)]
pub struct UpsertForWorkerParams<T1: crate::StringSql, T2: crate::StringSql, T3: crate::StringSql> {
    pub alias_id: i64,
    pub real_email: T1,
    pub token: T2,
    pub reply_address: T3,
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ListForAlias {
    pub id: i64,
    pub real_email: String,
    pub reply_address: String,
    pub display_name: Option<String>,
    pub enabled: bool,
    pub block_reply: bool,
    #[serde(with = "time::serde::rfc3339::option")]
    pub last_seen_at: Option<time::OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: time::OffsetDateTime,
}
pub struct ListForAliasBorrowed<'a> {
    pub id: i64,
    pub real_email: &'a str,
    pub reply_address: &'a str,
    pub display_name: Option<&'a str>,
    pub enabled: bool,
    pub block_reply: bool,
    pub last_seen_at: Option<time::OffsetDateTime>,
    pub created_at: time::OffsetDateTime,
}
impl<'a> From<ListForAliasBorrowed<'a>> for ListForAlias {
    fn from(
        ListForAliasBorrowed {
            id,
            real_email,
            reply_address,
            display_name,
            enabled,
            block_reply,
            last_seen_at,
            created_at,
        }: ListForAliasBorrowed<'a>,
    ) -> Self {
        Self {
            id,
            real_email: real_email.into(),
            reply_address: reply_address.into(),
            display_name: display_name.map(|v| v.into()),
            enabled,
            block_reply,
            last_seen_at,
            created_at,
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct UpsertForWorker {
    pub token: String,
    pub enabled: bool,
}
pub struct UpsertForWorkerBorrowed<'a> {
    pub token: &'a str,
    pub enabled: bool,
}
impl<'a> From<UpsertForWorkerBorrowed<'a>> for UpsertForWorker {
    fn from(UpsertForWorkerBorrowed { token, enabled }: UpsertForWorkerBorrowed<'a>) -> Self {
        Self {
            token: token.into(),
            enabled,
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ReplyJoin {
    pub real_email: String,
    pub rc_enabled: bool,
    pub block_reply: bool,
    pub alias_id: i64,
    pub alias_address: String,
    pub alias_enabled: bool,
    pub alias_domain: String,
    pub mailbox_email: String,
    pub mailbox_enabled: bool,
    pub user_enabled: bool,
}
pub struct ReplyJoinBorrowed<'a> {
    pub real_email: &'a str,
    pub rc_enabled: bool,
    pub block_reply: bool,
    pub alias_id: i64,
    pub alias_address: &'a str,
    pub alias_enabled: bool,
    pub alias_domain: &'a str,
    pub mailbox_email: &'a str,
    pub mailbox_enabled: bool,
    pub user_enabled: bool,
}
impl<'a> From<ReplyJoinBorrowed<'a>> for ReplyJoin {
    fn from(
        ReplyJoinBorrowed {
            real_email,
            rc_enabled,
            block_reply,
            alias_id,
            alias_address,
            alias_enabled,
            alias_domain,
            mailbox_email,
            mailbox_enabled,
            user_enabled,
        }: ReplyJoinBorrowed<'a>,
    ) -> Self {
        Self {
            real_email: real_email.into(),
            rc_enabled,
            block_reply,
            alias_id,
            alias_address: alias_address.into(),
            alias_enabled,
            alias_domain: alias_domain.into(),
            mailbox_email: mailbox_email.into(),
            mailbox_enabled,
            user_enabled,
        }
    }
}
use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
pub struct ListForAliasQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<ListForAliasBorrowed, tokio_postgres::Error>,
    mapper: fn(ListForAliasBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> ListForAliasQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(ListForAliasBorrowed) -> R,
    ) -> ListForAliasQuery<'c, 'a, 's, C, R, N> {
        ListForAliasQuery {
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
pub struct UpsertForWorkerQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<UpsertForWorkerBorrowed, tokio_postgres::Error>,
    mapper: fn(UpsertForWorkerBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> UpsertForWorkerQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(UpsertForWorkerBorrowed) -> R,
    ) -> UpsertForWorkerQuery<'c, 'a, 's, C, R, N> {
        UpsertForWorkerQuery {
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
pub struct ReplyJoinQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<ReplyJoinBorrowed, tokio_postgres::Error>,
    mapper: fn(ReplyJoinBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> ReplyJoinQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(self, mapper: fn(ReplyJoinBorrowed) -> R) -> ReplyJoinQuery<'c, 'a, 's, C, R, N> {
        ReplyJoinQuery {
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
pub struct ListForAliasStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn list_for_alias() -> ListForAliasStmt {
    ListForAliasStmt(
        "SELECT id, real_email::text AS real_email, reply_address::text AS reply_address, display_name, enabled, block_reply, last_seen_at, created_at FROM reverse_contact WHERE alias_id = $1 ORDER BY id DESC",
        None,
    )
}
impl ListForAliasStmt {
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
    ) -> ListForAliasQuery<'c, 'a, 's, C, ListForAlias, 1> {
        ListForAliasQuery {
            client,
            params: [alias_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<ListForAliasBorrowed, tokio_postgres::Error> {
                    Ok(ListForAliasBorrowed {
                        id: row.try_get(0)?,
                        real_email: row.try_get(1)?,
                        reply_address: row.try_get(2)?,
                        display_name: row.try_get(3)?,
                        enabled: row.try_get(4)?,
                        block_reply: row.try_get(5)?,
                        last_seen_at: row.try_get(6)?,
                        created_at: row.try_get(7)?,
                    })
                },
            mapper: |it| ListForAlias::from(it),
        }
    }
}
pub struct ExistsForUserStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn exists_for_user() -> ExistsForUserStmt {
    ExistsForUserStmt(
        "SELECT 1 AS one FROM reverse_contact rc JOIN alias a ON a.id = rc.alias_id WHERE rc.id = $1 AND a.user_id = $2",
        None,
    )
}
impl ExistsForUserStmt {
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
        contact_id: &'a i64,
        user_id: &'a i64,
    ) -> I32Query<'c, 'a, 's, C, i32, 2> {
        I32Query {
            client,
            params: [contact_id, user_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it,
        }
    }
}
impl<'c, 'a, 's, C: GenericClient>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        ExistsForUserParams,
        I32Query<'c, 'a, 's, C, i32, 2>,
        C,
    > for ExistsForUserStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a ExistsForUserParams,
    ) -> I32Query<'c, 'a, 's, C, i32, 2> {
        self.bind(client, &params.contact_id, &params.user_id)
    }
}
pub struct SetEnabledStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn set_enabled() -> SetEnabledStmt {
    SetEnabledStmt(
        "UPDATE reverse_contact SET enabled = $1 WHERE id = $2",
        None,
    )
}
impl SetEnabledStmt {
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
        enabled: &'a bool,
        contact_id: &'a i64,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[enabled, contact_id]).await
    }
}
impl<'a, C: GenericClient + Send + Sync>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        SetEnabledParams,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for SetEnabledStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a SetEnabledParams,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.enabled, &params.contact_id))
    }
}
pub struct SetBlockReplyStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn set_block_reply() -> SetBlockReplyStmt {
    SetBlockReplyStmt(
        "UPDATE reverse_contact SET block_reply = $1 WHERE id = $2",
        None,
    )
}
impl SetBlockReplyStmt {
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
        block_reply: &'a bool,
        contact_id: &'a i64,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[block_reply, contact_id]).await
    }
}
impl<'a, C: GenericClient + Send + Sync>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        SetBlockReplyParams,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for SetBlockReplyStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a SetBlockReplyParams,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.block_reply, &params.contact_id))
    }
}
pub struct SetDisplayNameStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn set_display_name() -> SetDisplayNameStmt {
    SetDisplayNameStmt(
        "UPDATE reverse_contact SET display_name = $1 WHERE id = $2",
        None,
    )
}
impl SetDisplayNameStmt {
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
        display_name: &'a Option<T1>,
        contact_id: &'a i64,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[display_name, contact_id]).await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::StringSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        SetDisplayNameParams<T1>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for SetDisplayNameStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a SetDisplayNameParams<T1>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.display_name, &params.contact_id))
    }
}
pub struct DeleteForUserStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn delete_for_user() -> DeleteForUserStmt {
    DeleteForUserStmt(
        "DELETE FROM reverse_contact rc USING alias a WHERE rc.id = $1 AND rc.alias_id = a.id AND a.user_id = $2",
        None,
    )
}
impl DeleteForUserStmt {
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
        contact_id: &'a i64,
        user_id: &'a i64,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[contact_id, user_id]).await
    }
}
impl<'a, C: GenericClient + Send + Sync>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        DeleteForUserParams,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for DeleteForUserStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a DeleteForUserParams,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.contact_id, &params.user_id))
    }
}
pub struct UpsertForWorkerStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn upsert_for_worker() -> UpsertForWorkerStmt {
    UpsertForWorkerStmt(
        "WITH ins AS ( INSERT INTO reverse_contact (alias_id, real_email, token, reply_address) VALUES ($1, $2, $3, $4) ON CONFLICT (alias_id, real_email) DO UPDATE SET last_seen_at = now() RETURNING token, enabled ) SELECT token, enabled FROM ins",
        None,
    )
}
impl UpsertForWorkerStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<
        'c,
        'a,
        's,
        C: GenericClient,
        T1: crate::StringSql,
        T2: crate::StringSql,
        T3: crate::StringSql,
    >(
        &'s self,
        client: &'c C,
        alias_id: &'a i64,
        real_email: &'a T1,
        token: &'a T2,
        reply_address: &'a T3,
    ) -> UpsertForWorkerQuery<'c, 'a, 's, C, UpsertForWorker, 4> {
        UpsertForWorkerQuery {
            client,
            params: [alias_id, real_email, token, reply_address],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<UpsertForWorkerBorrowed, tokio_postgres::Error> {
                Ok(UpsertForWorkerBorrowed {
                    token: row.try_get(0)?,
                    enabled: row.try_get(1)?,
                })
            },
            mapper: |it| UpsertForWorker::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::StringSql, T2: crate::StringSql, T3: crate::StringSql>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        UpsertForWorkerParams<T1, T2, T3>,
        UpsertForWorkerQuery<'c, 'a, 's, C, UpsertForWorker, 4>,
        C,
    > for UpsertForWorkerStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a UpsertForWorkerParams<T1, T2, T3>,
    ) -> UpsertForWorkerQuery<'c, 'a, 's, C, UpsertForWorker, 4> {
        self.bind(
            client,
            &params.alias_id,
            &params.real_email,
            &params.token,
            &params.reply_address,
        )
    }
}
pub struct ReplyJoinStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn reply_join() -> ReplyJoinStmt {
    ReplyJoinStmt(
        "SELECT rc.real_email::text AS real_email, rc.enabled AS rc_enabled, rc.block_reply, a.id AS alias_id, a.address::text AS alias_address, a.enabled AS alias_enabled, d.domain::text AS alias_domain, m.email::text AS mailbox_email, m.enabled AS mailbox_enabled, u.enabled AS user_enabled FROM reverse_contact rc JOIN alias a ON a.id = rc.alias_id JOIN alias_domain d ON d.id = a.domain_id JOIN mailbox m ON m.id = a.mailbox_id JOIN \"user\" u ON u.id = a.user_id WHERE rc.id = $1",
        None,
    )
}
impl ReplyJoinStmt {
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
        rc_id: &'a i64,
    ) -> ReplyJoinQuery<'c, 'a, 's, C, ReplyJoin, 1> {
        ReplyJoinQuery {
            client,
            params: [rc_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<ReplyJoinBorrowed, tokio_postgres::Error> {
                    Ok(ReplyJoinBorrowed {
                        real_email: row.try_get(0)?,
                        rc_enabled: row.try_get(1)?,
                        block_reply: row.try_get(2)?,
                        alias_id: row.try_get(3)?,
                        alias_address: row.try_get(4)?,
                        alias_enabled: row.try_get(5)?,
                        alias_domain: row.try_get(6)?,
                        mailbox_email: row.try_get(7)?,
                        mailbox_enabled: row.try_get(8)?,
                        user_enabled: row.try_get(9)?,
                    })
                },
            mapper: |it| ReplyJoin::from(it),
        }
    }
}
