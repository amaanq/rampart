// This file was generated with `cornucopia`. Do not modify.

#[derive(Debug)]
pub struct ListForUserFilteredParams<T1: crate::StringSql> {
    pub user_id: i64,
    pub query: Option<T1>,
    pub pinned: Option<bool>,
    pub lim: i64,
    pub off: i64,
}
#[derive(Clone, Copy, Debug)]
pub struct ByIdUserParams {
    pub alias_id: i64,
    pub user_id: i64,
}
#[derive(Clone, Copy, Debug)]
pub struct AddressForUserParams {
    pub alias_id: i64,
    pub user_id: i64,
}
#[derive(Clone, Copy, Debug)]
pub struct ExistsForUserParams {
    pub alias_id: i64,
    pub user_id: i64,
}
#[derive(Debug)]
pub struct SetNoteParams<T1: crate::StringSql> {
    pub note: Option<T1>,
    pub alias_id: i64,
    pub user_id: i64,
}
#[derive(Clone, Copy, Debug)]
pub struct SetPinnedParams {
    pub pinned: bool,
    pub alias_id: i64,
    pub user_id: i64,
}
#[derive(Clone, Copy, Debug)]
pub struct SetMailboxParams {
    pub mailbox_id: i64,
    pub alias_id: i64,
    pub user_id: i64,
}
#[derive(Clone, Copy, Debug)]
pub struct ToggleEnabledParams {
    pub alias_id: i64,
    pub user_id: i64,
}
#[derive(Clone, Copy, Debug)]
pub struct DeleteParams {
    pub alias_id: i64,
    pub user_id: i64,
}
#[derive(Debug)]
pub struct CreateParams<T1: crate::StringSql, T2: crate::StringSql> {
    pub user_id: i64,
    pub address: T1,
    pub domain_id: i64,
    pub mailbox_id: i64,
    pub note: Option<T2>,
    pub auto_created: bool,
}
#[derive(Debug)]
pub struct CreateWithFlagsParams<T1: crate::StringSql, T2: crate::StringSql> {
    pub user_id: i64,
    pub address: T1,
    pub domain_id: i64,
    pub mailbox_id: i64,
    pub enabled: bool,
    pub pinned: bool,
    pub note: Option<T2>,
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct AliasJoinedRow {
    pub id: i64,
    pub address: String,
    pub enabled: bool,
    pub note: Option<String>,
    pub pinned: bool,
    pub nb_forward: i64,
    pub nb_block: i64,
    pub nb_reply: i64,
    pub mailbox_id: i64,
    pub mailbox_email: String,
    pub domain: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: time::OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub last_email_at: Option<time::OffsetDateTime>,
}
pub struct AliasJoinedRowBorrowed<'a> {
    pub id: i64,
    pub address: &'a str,
    pub enabled: bool,
    pub note: Option<&'a str>,
    pub pinned: bool,
    pub nb_forward: i64,
    pub nb_block: i64,
    pub nb_reply: i64,
    pub mailbox_id: i64,
    pub mailbox_email: &'a str,
    pub domain: &'a str,
    pub created_at: time::OffsetDateTime,
    pub last_email_at: Option<time::OffsetDateTime>,
}
impl<'a> From<AliasJoinedRowBorrowed<'a>> for AliasJoinedRow {
    fn from(
        AliasJoinedRowBorrowed {
            id,
            address,
            enabled,
            note,
            pinned,
            nb_forward,
            nb_block,
            nb_reply,
            mailbox_id,
            mailbox_email,
            domain,
            created_at,
            last_email_at,
        }: AliasJoinedRowBorrowed<'a>,
    ) -> Self {
        Self {
            id,
            address: address.into(),
            enabled,
            note: note.map(|v| v.into()),
            pinned,
            nb_forward,
            nb_block,
            nb_reply,
            mailbox_id,
            mailbox_email: mailbox_email.into(),
            domain: domain.into(),
            created_at,
            last_email_at,
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ListForDashboard {
    pub id: i64,
    pub address: String,
    pub enabled: bool,
    pub note: Option<String>,
    pub pinned: bool,
    pub nb_forward: i64,
    pub nb_block: i64,
    pub nb_reply: i64,
    pub mailbox_id: i64,
    pub mailbox_email: String,
    pub domain: String,
    #[serde(with = "time::serde::rfc3339::option")]
    pub last_email_at: Option<time::OffsetDateTime>,
}
pub struct ListForDashboardBorrowed<'a> {
    pub id: i64,
    pub address: &'a str,
    pub enabled: bool,
    pub note: Option<&'a str>,
    pub pinned: bool,
    pub nb_forward: i64,
    pub nb_block: i64,
    pub nb_reply: i64,
    pub mailbox_id: i64,
    pub mailbox_email: &'a str,
    pub domain: &'a str,
    pub last_email_at: Option<time::OffsetDateTime>,
}
impl<'a> From<ListForDashboardBorrowed<'a>> for ListForDashboard {
    fn from(
        ListForDashboardBorrowed {
            id,
            address,
            enabled,
            note,
            pinned,
            nb_forward,
            nb_block,
            nb_reply,
            mailbox_id,
            mailbox_email,
            domain,
            last_email_at,
        }: ListForDashboardBorrowed<'a>,
    ) -> Self {
        Self {
            id,
            address: address.into(),
            enabled,
            note: note.map(|v| v.into()),
            pinned,
            nb_forward,
            nb_block,
            nb_reply,
            mailbox_id,
            mailbox_email: mailbox_email.into(),
            domain: domain.into(),
            last_email_at,
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct AliasExportRow {
    pub address: String,
    pub note: Option<String>,
    pub enabled: bool,
    pub pinned: bool,
    pub mailbox: String,
    pub user_email: String,
}
pub struct AliasExportRowBorrowed<'a> {
    pub address: &'a str,
    pub note: Option<&'a str>,
    pub enabled: bool,
    pub pinned: bool,
    pub mailbox: &'a str,
    pub user_email: &'a str,
}
impl<'a> From<AliasExportRowBorrowed<'a>> for AliasExportRow {
    fn from(
        AliasExportRowBorrowed {
            address,
            note,
            enabled,
            pinned,
            mailbox,
            user_email,
        }: AliasExportRowBorrowed<'a>,
    ) -> Self {
        Self {
            address: address.into(),
            note: note.map(|v| v.into()),
            enabled,
            pinned,
            mailbox: mailbox.into(),
            user_email: user_email.into(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ForwardJoin {
    pub alias_address: String,
    pub alias_enabled: bool,
    pub mailbox_email: String,
    pub mailbox_enabled: bool,
    pub user_enabled: bool,
    pub alias_domain: String,
    pub user_id: i64,
}
pub struct ForwardJoinBorrowed<'a> {
    pub alias_address: &'a str,
    pub alias_enabled: bool,
    pub mailbox_email: &'a str,
    pub mailbox_enabled: bool,
    pub user_enabled: bool,
    pub alias_domain: &'a str,
    pub user_id: i64,
}
impl<'a> From<ForwardJoinBorrowed<'a>> for ForwardJoin {
    fn from(
        ForwardJoinBorrowed {
            alias_address,
            alias_enabled,
            mailbox_email,
            mailbox_enabled,
            user_enabled,
            alias_domain,
            user_id,
        }: ForwardJoinBorrowed<'a>,
    ) -> Self {
        Self {
            alias_address: alias_address.into(),
            alias_enabled,
            mailbox_email: mailbox_email.into(),
            mailbox_enabled,
            user_enabled,
            alias_domain: alias_domain.into(),
            user_id,
        }
    }
}
use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
pub struct AliasJoinedRowQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<AliasJoinedRowBorrowed, tokio_postgres::Error>,
    mapper: fn(AliasJoinedRowBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> AliasJoinedRowQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(AliasJoinedRowBorrowed) -> R,
    ) -> AliasJoinedRowQuery<'c, 'a, 's, C, R, N> {
        AliasJoinedRowQuery {
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
pub struct ListForDashboardQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<ListForDashboardBorrowed, tokio_postgres::Error>,
    mapper: fn(ListForDashboardBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> ListForDashboardQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(ListForDashboardBorrowed) -> R,
    ) -> ListForDashboardQuery<'c, 'a, 's, C, R, N> {
        ListForDashboardQuery {
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
pub struct StringQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<&str, tokio_postgres::Error>,
    mapper: fn(&str) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> StringQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(self, mapper: fn(&str) -> R) -> StringQuery<'c, 'a, 's, C, R, N> {
        StringQuery {
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
pub struct AliasExportRowQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<AliasExportRowBorrowed, tokio_postgres::Error>,
    mapper: fn(AliasExportRowBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> AliasExportRowQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(AliasExportRowBorrowed) -> R,
    ) -> AliasExportRowQuery<'c, 'a, 's, C, R, N> {
        AliasExportRowQuery {
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
pub struct ForwardJoinQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<ForwardJoinBorrowed, tokio_postgres::Error>,
    mapper: fn(ForwardJoinBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> ForwardJoinQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(ForwardJoinBorrowed) -> R,
    ) -> ForwardJoinQuery<'c, 'a, 's, C, R, N> {
        ForwardJoinQuery {
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
pub struct ListForUserFilteredStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn list_for_user_filtered() -> ListForUserFilteredStmt {
    ListForUserFilteredStmt(
        "SELECT a.id, a.address::text AS address, a.enabled, a.note, a.pinned, a.nb_forward, a.nb_block, a.nb_reply, a.mailbox_id, m.email::text AS mailbox_email, d.domain::text AS domain, a.created_at, a.last_email_at FROM alias a JOIN mailbox m ON m.id = a.mailbox_id JOIN alias_domain d ON d.id = a.domain_id WHERE a.user_id = $1 AND ($2::text IS NULL OR a.address::text ILIKE $2 OR a.note ILIKE $2) AND ($3::bool IS NULL OR a.pinned = $3) ORDER BY a.pinned DESC, a.last_email_at DESC NULLS LAST, a.id DESC LIMIT $4 OFFSET $5",
        None,
    )
}
impl ListForUserFilteredStmt {
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
        user_id: &'a i64,
        query: &'a Option<T1>,
        pinned: &'a Option<bool>,
        lim: &'a i64,
        off: &'a i64,
    ) -> AliasJoinedRowQuery<'c, 'a, 's, C, AliasJoinedRow, 5> {
        AliasJoinedRowQuery {
            client,
            params: [user_id, query, pinned, lim, off],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<AliasJoinedRowBorrowed, tokio_postgres::Error> {
                Ok(AliasJoinedRowBorrowed {
                    id: row.try_get(0)?,
                    address: row.try_get(1)?,
                    enabled: row.try_get(2)?,
                    note: row.try_get(3)?,
                    pinned: row.try_get(4)?,
                    nb_forward: row.try_get(5)?,
                    nb_block: row.try_get(6)?,
                    nb_reply: row.try_get(7)?,
                    mailbox_id: row.try_get(8)?,
                    mailbox_email: row.try_get(9)?,
                    domain: row.try_get(10)?,
                    created_at: row.try_get(11)?,
                    last_email_at: row.try_get(12)?,
                })
            },
            mapper: |it| AliasJoinedRow::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        ListForUserFilteredParams<T1>,
        AliasJoinedRowQuery<'c, 'a, 's, C, AliasJoinedRow, 5>,
        C,
    > for ListForUserFilteredStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a ListForUserFilteredParams<T1>,
    ) -> AliasJoinedRowQuery<'c, 'a, 's, C, AliasJoinedRow, 5> {
        self.bind(
            client,
            &params.user_id,
            &params.query,
            &params.pinned,
            &params.lim,
            &params.off,
        )
    }
}
pub struct ByIdUserStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn by_id_user() -> ByIdUserStmt {
    ByIdUserStmt(
        "SELECT a.id, a.address::text AS address, a.enabled, a.note, a.pinned, a.nb_forward, a.nb_block, a.nb_reply, a.mailbox_id, m.email::text AS mailbox_email, d.domain::text AS domain, a.created_at, a.last_email_at FROM alias a JOIN mailbox m ON m.id = a.mailbox_id JOIN alias_domain d ON d.id = a.domain_id WHERE a.id = $1 AND a.user_id = $2",
        None,
    )
}
impl ByIdUserStmt {
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
        user_id: &'a i64,
    ) -> AliasJoinedRowQuery<'c, 'a, 's, C, AliasJoinedRow, 2> {
        AliasJoinedRowQuery {
            client,
            params: [alias_id, user_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<AliasJoinedRowBorrowed, tokio_postgres::Error> {
                Ok(AliasJoinedRowBorrowed {
                    id: row.try_get(0)?,
                    address: row.try_get(1)?,
                    enabled: row.try_get(2)?,
                    note: row.try_get(3)?,
                    pinned: row.try_get(4)?,
                    nb_forward: row.try_get(5)?,
                    nb_block: row.try_get(6)?,
                    nb_reply: row.try_get(7)?,
                    mailbox_id: row.try_get(8)?,
                    mailbox_email: row.try_get(9)?,
                    domain: row.try_get(10)?,
                    created_at: row.try_get(11)?,
                    last_email_at: row.try_get(12)?,
                })
            },
            mapper: |it| AliasJoinedRow::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        ByIdUserParams,
        AliasJoinedRowQuery<'c, 'a, 's, C, AliasJoinedRow, 2>,
        C,
    > for ByIdUserStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a ByIdUserParams,
    ) -> AliasJoinedRowQuery<'c, 'a, 's, C, AliasJoinedRow, 2> {
        self.bind(client, &params.alias_id, &params.user_id)
    }
}
pub struct ByIdStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn by_id() -> ByIdStmt {
    ByIdStmt(
        "SELECT a.id, a.address::text AS address, a.enabled, a.note, a.pinned, a.nb_forward, a.nb_block, a.nb_reply, a.mailbox_id, m.email::text AS mailbox_email, d.domain::text AS domain, a.created_at, a.last_email_at FROM alias a JOIN mailbox m ON m.id = a.mailbox_id JOIN alias_domain d ON d.id = a.domain_id WHERE a.id = $1",
        None,
    )
}
impl ByIdStmt {
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
    ) -> AliasJoinedRowQuery<'c, 'a, 's, C, AliasJoinedRow, 1> {
        AliasJoinedRowQuery {
            client,
            params: [alias_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<AliasJoinedRowBorrowed, tokio_postgres::Error> {
                Ok(AliasJoinedRowBorrowed {
                    id: row.try_get(0)?,
                    address: row.try_get(1)?,
                    enabled: row.try_get(2)?,
                    note: row.try_get(3)?,
                    pinned: row.try_get(4)?,
                    nb_forward: row.try_get(5)?,
                    nb_block: row.try_get(6)?,
                    nb_reply: row.try_get(7)?,
                    mailbox_id: row.try_get(8)?,
                    mailbox_email: row.try_get(9)?,
                    domain: row.try_get(10)?,
                    created_at: row.try_get(11)?,
                    last_email_at: row.try_get(12)?,
                })
            },
            mapper: |it| AliasJoinedRow::from(it),
        }
    }
}
pub struct ListForDashboardStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn list_for_dashboard() -> ListForDashboardStmt {
    ListForDashboardStmt(
        "SELECT a.id, a.address::text AS address, a.enabled, a.note, a.pinned, a.nb_forward, a.nb_block, a.nb_reply, a.mailbox_id, m.email::text AS mailbox_email, d.domain::text AS domain, a.last_email_at FROM alias a JOIN mailbox m ON m.id = a.mailbox_id JOIN alias_domain d ON d.id = a.domain_id WHERE a.user_id = $1 ORDER BY a.pinned DESC, a.last_email_at DESC NULLS LAST, a.id DESC LIMIT 200",
        None,
    )
}
impl ListForDashboardStmt {
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
        user_id: &'a i64,
    ) -> ListForDashboardQuery<'c, 'a, 's, C, ListForDashboard, 1> {
        ListForDashboardQuery {
            client,
            params: [user_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<ListForDashboardBorrowed, tokio_postgres::Error> {
                Ok(ListForDashboardBorrowed {
                    id: row.try_get(0)?,
                    address: row.try_get(1)?,
                    enabled: row.try_get(2)?,
                    note: row.try_get(3)?,
                    pinned: row.try_get(4)?,
                    nb_forward: row.try_get(5)?,
                    nb_block: row.try_get(6)?,
                    nb_reply: row.try_get(7)?,
                    mailbox_id: row.try_get(8)?,
                    mailbox_email: row.try_get(9)?,
                    domain: row.try_get(10)?,
                    last_email_at: row.try_get(11)?,
                })
            },
            mapper: |it| ListForDashboard::from(it),
        }
    }
}
pub struct AddressForUserStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn address_for_user() -> AddressForUserStmt {
    AddressForUserStmt(
        "SELECT a.address::text AS address FROM alias a WHERE a.id = $1 AND a.user_id = $2",
        None,
    )
}
impl AddressForUserStmt {
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
        user_id: &'a i64,
    ) -> StringQuery<'c, 'a, 's, C, String, 2> {
        StringQuery {
            client,
            params: [alias_id, user_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it.into(),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        AddressForUserParams,
        StringQuery<'c, 'a, 's, C, String, 2>,
        C,
    > for AddressForUserStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a AddressForUserParams,
    ) -> StringQuery<'c, 'a, 's, C, String, 2> {
        self.bind(client, &params.alias_id, &params.user_id)
    }
}
pub struct ExistsForUserStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn exists_for_user() -> ExistsForUserStmt {
    ExistsForUserStmt(
        "SELECT 1 AS one FROM alias WHERE id = $1 AND user_id = $2",
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
        alias_id: &'a i64,
        user_id: &'a i64,
    ) -> I32Query<'c, 'a, 's, C, i32, 2> {
        I32Query {
            client,
            params: [alias_id, user_id],
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
        self.bind(client, &params.alias_id, &params.user_id)
    }
}
pub struct SetNoteStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn set_note() -> SetNoteStmt {
    SetNoteStmt(
        "UPDATE alias SET note = $1 WHERE id = $2 AND user_id = $3",
        None,
    )
}
impl SetNoteStmt {
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
        note: &'a Option<T1>,
        alias_id: &'a i64,
        user_id: &'a i64,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[note, alias_id, user_id]).await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::StringSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        SetNoteParams<T1>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for SetNoteStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a SetNoteParams<T1>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.note, &params.alias_id, &params.user_id))
    }
}
pub struct SetPinnedStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn set_pinned() -> SetPinnedStmt {
    SetPinnedStmt(
        "UPDATE alias SET pinned = $1 WHERE id = $2 AND user_id = $3",
        None,
    )
}
impl SetPinnedStmt {
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
        pinned: &'a bool,
        alias_id: &'a i64,
        user_id: &'a i64,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[pinned, alias_id, user_id]).await
    }
}
impl<'a, C: GenericClient + Send + Sync>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        SetPinnedParams,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for SetPinnedStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a SetPinnedParams,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.pinned, &params.alias_id, &params.user_id))
    }
}
pub struct SetMailboxStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn set_mailbox() -> SetMailboxStmt {
    SetMailboxStmt(
        "UPDATE alias SET mailbox_id = $1 WHERE id = $2 AND user_id = $3",
        None,
    )
}
impl SetMailboxStmt {
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
        mailbox_id: &'a i64,
        alias_id: &'a i64,
        user_id: &'a i64,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(self.0, &[mailbox_id, alias_id, user_id])
            .await
    }
}
impl<'a, C: GenericClient + Send + Sync>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        SetMailboxParams,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for SetMailboxStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a SetMailboxParams,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.mailbox_id,
            &params.alias_id,
            &params.user_id,
        ))
    }
}
pub struct ToggleEnabledStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn toggle_enabled() -> ToggleEnabledStmt {
    ToggleEnabledStmt(
        "UPDATE alias SET enabled = NOT enabled WHERE id = $1 AND user_id = $2",
        None,
    )
}
impl ToggleEnabledStmt {
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
        alias_id: &'a i64,
        user_id: &'a i64,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[alias_id, user_id]).await
    }
}
impl<'a, C: GenericClient + Send + Sync>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        ToggleEnabledParams,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for ToggleEnabledStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a ToggleEnabledParams,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.alias_id, &params.user_id))
    }
}
pub struct DeleteStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn delete() -> DeleteStmt {
    DeleteStmt("DELETE FROM alias WHERE id = $1 AND user_id = $2", None)
}
impl DeleteStmt {
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
        alias_id: &'a i64,
        user_id: &'a i64,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[alias_id, user_id]).await
    }
}
impl<'a, C: GenericClient + Send + Sync>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        DeleteParams,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for DeleteStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a DeleteParams,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.alias_id, &params.user_id))
    }
}
pub struct DisableAllForUserStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn disable_all_for_user() -> DisableAllForUserStmt {
    DisableAllForUserStmt("UPDATE alias SET enabled = FALSE WHERE user_id = $1", None)
}
impl DisableAllForUserStmt {
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
        user_id: &'a i64,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[user_id]).await
    }
}
pub struct CreateStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn create() -> CreateStmt {
    CreateStmt(
        "INSERT INTO alias (user_id, address, domain_id, mailbox_id, note, auto_created) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
        None,
    )
}
impl CreateStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql, T2: crate::StringSql>(
        &'s self,
        client: &'c C,
        user_id: &'a i64,
        address: &'a T1,
        domain_id: &'a i64,
        mailbox_id: &'a i64,
        note: &'a Option<T2>,
        auto_created: &'a bool,
    ) -> I64Query<'c, 'a, 's, C, i64, 6> {
        I64Query {
            client,
            params: [user_id, address, domain_id, mailbox_id, note, auto_created],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it,
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::StringSql, T2: crate::StringSql>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        CreateParams<T1, T2>,
        I64Query<'c, 'a, 's, C, i64, 6>,
        C,
    > for CreateStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a CreateParams<T1, T2>,
    ) -> I64Query<'c, 'a, 's, C, i64, 6> {
        self.bind(
            client,
            &params.user_id,
            &params.address,
            &params.domain_id,
            &params.mailbox_id,
            &params.note,
            &params.auto_created,
        )
    }
}
pub struct CreateWithFlagsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn create_with_flags() -> CreateWithFlagsStmt {
    CreateWithFlagsStmt(
        "INSERT INTO alias (user_id, address, domain_id, mailbox_id, enabled, pinned, note) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        None,
    )
}
impl CreateWithFlagsStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql, T2: crate::StringSql>(
        &'s self,
        client: &'c C,
        user_id: &'a i64,
        address: &'a T1,
        domain_id: &'a i64,
        mailbox_id: &'a i64,
        enabled: &'a bool,
        pinned: &'a bool,
        note: &'a Option<T2>,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(
                self.0,
                &[
                    user_id, address, domain_id, mailbox_id, enabled, pinned, note,
                ],
            )
            .await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::StringSql, T2: crate::StringSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        CreateWithFlagsParams<T1, T2>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for CreateWithFlagsStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a CreateWithFlagsParams<T1, T2>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.user_id,
            &params.address,
            &params.domain_id,
            &params.mailbox_id,
            &params.enabled,
            &params.pinned,
            &params.note,
        ))
    }
}
pub struct BumpForwardCountStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn bump_forward_count() -> BumpForwardCountStmt {
    BumpForwardCountStmt(
        "UPDATE alias SET nb_forward = nb_forward + 1, last_email_at = now() WHERE id = $1",
        None,
    )
}
impl BumpForwardCountStmt {
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
        alias_id: &'a i64,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[alias_id]).await
    }
}
pub struct BumpBlockCountStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn bump_block_count() -> BumpBlockCountStmt {
    BumpBlockCountStmt(
        "UPDATE alias SET nb_block = nb_block + 1, last_email_at = now() WHERE id = $1",
        None,
    )
}
impl BumpBlockCountStmt {
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
        alias_id: &'a i64,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[alias_id]).await
    }
}
pub struct ExportStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn export() -> ExportStmt {
    ExportStmt(
        "SELECT a.address::text AS address, a.note, a.enabled, a.pinned, m.email::text AS mailbox, u.email::text AS user_email FROM alias a JOIN mailbox m ON m.id = a.mailbox_id JOIN \"user\" u ON u.id = a.user_id ORDER BY u.id, a.id",
        None,
    )
}
impl ExportStmt {
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
    ) -> AliasExportRowQuery<'c, 'a, 's, C, AliasExportRow, 0> {
        AliasExportRowQuery {
            client,
            params: [],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<AliasExportRowBorrowed, tokio_postgres::Error> {
                Ok(AliasExportRowBorrowed {
                    address: row.try_get(0)?,
                    note: row.try_get(1)?,
                    enabled: row.try_get(2)?,
                    pinned: row.try_get(3)?,
                    mailbox: row.try_get(4)?,
                    user_email: row.try_get(5)?,
                })
            },
            mapper: |it| AliasExportRow::from(it),
        }
    }
}
pub struct ExportForUserStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn export_for_user() -> ExportForUserStmt {
    ExportForUserStmt(
        "SELECT a.address::text AS address, a.note, a.enabled, a.pinned, m.email::text AS mailbox, u.email::text AS user_email FROM alias a JOIN mailbox m ON m.id = a.mailbox_id JOIN \"user\" u ON u.id = a.user_id WHERE u.email = $1 ORDER BY a.id",
        None,
    )
}
impl ExportForUserStmt {
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
        user_email: &'a T1,
    ) -> AliasExportRowQuery<'c, 'a, 's, C, AliasExportRow, 1> {
        AliasExportRowQuery {
            client,
            params: [user_email],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<AliasExportRowBorrowed, tokio_postgres::Error> {
                Ok(AliasExportRowBorrowed {
                    address: row.try_get(0)?,
                    note: row.try_get(1)?,
                    enabled: row.try_get(2)?,
                    pinned: row.try_get(3)?,
                    mailbox: row.try_get(4)?,
                    user_email: row.try_get(5)?,
                })
            },
            mapper: |it| AliasExportRow::from(it),
        }
    }
}
pub struct ForwardJoinStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn forward_join() -> ForwardJoinStmt {
    ForwardJoinStmt(
        "SELECT a.address::text AS alias_address, a.enabled AS alias_enabled, m.email::text AS mailbox_email, m.enabled AS mailbox_enabled, u.enabled AS user_enabled, d.domain::text AS alias_domain, a.user_id FROM alias a JOIN mailbox m ON m.id = a.mailbox_id JOIN \"user\" u ON u.id = a.user_id JOIN alias_domain d ON d.id = a.domain_id WHERE a.id = $1",
        None,
    )
}
impl ForwardJoinStmt {
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
    ) -> ForwardJoinQuery<'c, 'a, 's, C, ForwardJoin, 1> {
        ForwardJoinQuery {
            client,
            params: [alias_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<ForwardJoinBorrowed, tokio_postgres::Error> {
                    Ok(ForwardJoinBorrowed {
                        alias_address: row.try_get(0)?,
                        alias_enabled: row.try_get(1)?,
                        mailbox_email: row.try_get(2)?,
                        mailbox_enabled: row.try_get(3)?,
                        user_enabled: row.try_get(4)?,
                        alias_domain: row.try_get(5)?,
                        user_id: row.try_get(6)?,
                    })
                },
            mapper: |it| ForwardJoin::from(it),
        }
    }
}
