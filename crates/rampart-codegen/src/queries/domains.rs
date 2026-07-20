// This file was generated with `cornucopia`. Do not modify.

#[derive(Clone, Copy, Debug)]
pub struct ListForUserParams {
    pub user_id: i64,
    pub is_admin: bool,
}
#[derive(Clone, Copy, Debug)]
pub struct ByIdForUserParams {
    pub domain_id: i64,
    pub user_id: i64,
    pub is_admin: bool,
}
#[derive(Clone, Copy, Debug)]
pub struct ListForDashboardParams {
    pub user_id: i64,
    pub is_admin: bool,
}
#[derive(Clone, Copy, Debug)]
pub struct ExistsManagableParams {
    pub domain_id: i64,
    pub user_id: i64,
    pub is_admin: bool,
}
#[derive(Clone, Copy, Debug)]
pub struct SetCatchAllAndCapParams {
    pub catch_all: bool,
    pub max_auto_created: Option<i32>,
    pub domain_id: i64,
}
#[derive(Debug)]
pub struct SetRandomPrefixParams<T1: crate::StringSql> {
    pub random_prefix: T1,
    pub domain_id: i64,
}
#[derive(Clone, Copy, Debug)]
pub struct SetDefaultMailboxParams {
    pub default_mailbox_id: Option<i64>,
    pub domain_id: i64,
}
#[derive(Debug)]
pub struct SetDkimRecordsParams<T1: crate::JsonSql> {
    pub dkim_records: T1,
    pub domain_id: i64,
}
#[derive(Debug)]
pub struct SetDnsCheckParams<T1: crate::JsonSql> {
    pub dns_status: T1,
    pub checked_at: time::OffsetDateTime,
    pub all_verified: bool,
    pub domain_id: i64,
}
#[derive(Clone, Copy, Debug)]
pub struct SetSharedParams {
    pub shared: bool,
    pub domain_id: i64,
}
#[derive(Clone, Copy, Debug)]
pub struct DeleteParams {
    pub domain_id: i64,
    pub user_id: i64,
    pub is_admin: bool,
}
#[derive(Debug)]
pub struct CreateParams<T1: crate::StringSql, T2: crate::StringSql> {
    pub domain: T1,
    pub owner_id: Option<i64>,
    pub random_prefix: Option<T2>,
}
#[derive(Clone, Copy, Debug)]
pub struct ByDomainForUserParams<T1: crate::StringSql> {
    pub domain: T1,
    pub user_id: i64,
    pub is_admin: bool,
}
#[derive(Clone, Copy, Debug)]
pub struct FirstAccessibleForUserParams {
    pub user_id: i64,
    pub is_admin: bool,
}
#[derive(Clone, Copy, Debug)]
pub struct SetDefaultMailboxByOwnerEmailParams<T1: crate::StringSql, T2: crate::StringSql> {
    pub mailbox_email: T1,
    pub domain: T2,
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct DomainRow {
    pub id: i64,
    pub domain: String,
    pub owner_id: Option<i64>,
    pub shared: bool,
    pub catch_all: bool,
    pub random_prefix: String,
    pub reply_prefix: String,
    pub default_mailbox_id: Option<i64>,
    pub dkim_records: serde_json::Value,
    pub dns_status: serde_json::Value,
    #[serde(with = "time::serde::rfc3339::option")]
    pub dns_checked_at: Option<time::OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub dns_verified_at: Option<time::OffsetDateTime>,
    pub nb_alias: i64,
}
pub struct DomainRowBorrowed<'a> {
    pub id: i64,
    pub domain: &'a str,
    pub owner_id: Option<i64>,
    pub shared: bool,
    pub catch_all: bool,
    pub random_prefix: &'a str,
    pub reply_prefix: &'a str,
    pub default_mailbox_id: Option<i64>,
    pub dkim_records: postgres_types::Json<&'a serde_json::value::RawValue>,
    pub dns_status: postgres_types::Json<&'a serde_json::value::RawValue>,
    pub dns_checked_at: Option<time::OffsetDateTime>,
    pub dns_verified_at: Option<time::OffsetDateTime>,
    pub nb_alias: i64,
}
impl<'a> From<DomainRowBorrowed<'a>> for DomainRow {
    fn from(
        DomainRowBorrowed {
            id,
            domain,
            owner_id,
            shared,
            catch_all,
            random_prefix,
            reply_prefix,
            default_mailbox_id,
            dkim_records,
            dns_status,
            dns_checked_at,
            dns_verified_at,
            nb_alias,
        }: DomainRowBorrowed<'a>,
    ) -> Self {
        Self {
            id,
            domain: domain.into(),
            owner_id,
            shared,
            catch_all,
            random_prefix: random_prefix.into(),
            reply_prefix: reply_prefix.into(),
            default_mailbox_id,
            dkim_records: serde_json::from_str(dkim_records.0.get()).unwrap(),
            dns_status: serde_json::from_str(dns_status.0.get()).unwrap(),
            dns_checked_at,
            dns_verified_at,
            nb_alias,
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ListForDashboard {
    pub id: i64,
    pub domain: String,
    pub shared: bool,
    pub owner_id: Option<i64>,
    pub random_prefix: String,
    pub reply_prefix: String,
    pub dkim_records: serde_json::Value,
    pub dns_status: serde_json::Value,
    #[serde(with = "time::serde::rfc3339::option")]
    pub dns_checked_at: Option<time::OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub dns_verified_at: Option<time::OffsetDateTime>,
    pub nb_alias: i64,
}
pub struct ListForDashboardBorrowed<'a> {
    pub id: i64,
    pub domain: &'a str,
    pub shared: bool,
    pub owner_id: Option<i64>,
    pub random_prefix: &'a str,
    pub reply_prefix: &'a str,
    pub dkim_records: postgres_types::Json<&'a serde_json::value::RawValue>,
    pub dns_status: postgres_types::Json<&'a serde_json::value::RawValue>,
    pub dns_checked_at: Option<time::OffsetDateTime>,
    pub dns_verified_at: Option<time::OffsetDateTime>,
    pub nb_alias: i64,
}
impl<'a> From<ListForDashboardBorrowed<'a>> for ListForDashboard {
    fn from(
        ListForDashboardBorrowed {
            id,
            domain,
            shared,
            owner_id,
            random_prefix,
            reply_prefix,
            dkim_records,
            dns_status,
            dns_checked_at,
            dns_verified_at,
            nb_alias,
        }: ListForDashboardBorrowed<'a>,
    ) -> Self {
        Self {
            id,
            domain: domain.into(),
            shared,
            owner_id,
            random_prefix: random_prefix.into(),
            reply_prefix: reply_prefix.into(),
            dkim_records: serde_json::from_str(dkim_records.0.get()).unwrap(),
            dns_status: serde_json::from_str(dns_status.0.get()).unwrap(),
            dns_checked_at,
            dns_verified_at,
            nb_alias,
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ListAdmin {
    pub id: i64,
    pub domain: String,
    pub shared: bool,
    pub owner_email: Option<String>,
    pub nb_alias: i64,
}
pub struct ListAdminBorrowed<'a> {
    pub id: i64,
    pub domain: &'a str,
    pub shared: bool,
    pub owner_email: Option<&'a str>,
    pub nb_alias: i64,
}
impl<'a> From<ListAdminBorrowed<'a>> for ListAdmin {
    fn from(
        ListAdminBorrowed {
            id,
            domain,
            shared,
            owner_email,
            nb_alias,
        }: ListAdminBorrowed<'a>,
    ) -> Self {
        Self {
            id,
            domain: domain.into(),
            shared,
            owner_email: owner_email.map(|v| v.into()),
            nb_alias,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Copy, serde::Serialize)]
pub struct CatchAllAndCap {
    pub catch_all: bool,
    pub max_auto_created: Option<i32>,
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct AliasDomainRow {
    pub id: i64,
    pub domain: String,
    pub owner_id: Option<i64>,
    pub shared: bool,
    pub catch_all: bool,
    pub random_prefix: String,
    pub reply_prefix: String,
    pub default_mailbox_id: Option<i64>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: time::OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: time::OffsetDateTime,
}
pub struct AliasDomainRowBorrowed<'a> {
    pub id: i64,
    pub domain: &'a str,
    pub owner_id: Option<i64>,
    pub shared: bool,
    pub catch_all: bool,
    pub random_prefix: &'a str,
    pub reply_prefix: &'a str,
    pub default_mailbox_id: Option<i64>,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}
impl<'a> From<AliasDomainRowBorrowed<'a>> for AliasDomainRow {
    fn from(
        AliasDomainRowBorrowed {
            id,
            domain,
            owner_id,
            shared,
            catch_all,
            random_prefix,
            reply_prefix,
            default_mailbox_id,
            created_at,
            updated_at,
        }: AliasDomainRowBorrowed<'a>,
    ) -> Self {
        Self {
            id,
            domain: domain.into(),
            owner_id,
            shared,
            catch_all,
            random_prefix: random_prefix.into(),
            reply_prefix: reply_prefix.into(),
            default_mailbox_id,
            created_at,
            updated_at,
        }
    }
}
use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
pub struct DomainRowQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<DomainRowBorrowed, tokio_postgres::Error>,
    mapper: fn(DomainRowBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> DomainRowQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(self, mapper: fn(DomainRowBorrowed) -> R) -> DomainRowQuery<'c, 'a, 's, C, R, N> {
        DomainRowQuery {
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
pub struct ListAdminQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<ListAdminBorrowed, tokio_postgres::Error>,
    mapper: fn(ListAdminBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> ListAdminQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(self, mapper: fn(ListAdminBorrowed) -> R) -> ListAdminQuery<'c, 'a, 's, C, R, N> {
        ListAdminQuery {
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
pub struct CatchAllAndCapQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<CatchAllAndCap, tokio_postgres::Error>,
    mapper: fn(CatchAllAndCap) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> CatchAllAndCapQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(CatchAllAndCap) -> R,
    ) -> CatchAllAndCapQuery<'c, 'a, 's, C, R, N> {
        CatchAllAndCapQuery {
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
pub struct AliasDomainRowQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<AliasDomainRowBorrowed, tokio_postgres::Error>,
    mapper: fn(AliasDomainRowBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> AliasDomainRowQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(AliasDomainRowBorrowed) -> R,
    ) -> AliasDomainRowQuery<'c, 'a, 's, C, R, N> {
        AliasDomainRowQuery {
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
pub struct OptionboolQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<Option<bool>, tokio_postgres::Error>,
    mapper: fn(Option<bool>) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> OptionboolQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(self, mapper: fn(Option<bool>) -> R) -> OptionboolQuery<'c, 'a, 's, C, R, N> {
        OptionboolQuery {
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
pub struct ListForUserStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn list_for_user() -> ListForUserStmt {
    ListForUserStmt(
        "SELECT d.id, d.domain::text AS domain, d.owner_id, d.shared, d.catch_all, d.random_prefix, d.reply_prefix, d.default_mailbox_id, d.dkim_records, d.dns_status, d.dns_checked_at, d.dns_verified_at, (SELECT COUNT(*) FROM alias a WHERE a.domain_id = d.id) AS nb_alias FROM alias_domain d WHERE d.shared OR d.owner_id = $1 OR $2 ORDER BY d.id",
        None,
    )
}
impl ListForUserStmt {
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
        is_admin: &'a bool,
    ) -> DomainRowQuery<'c, 'a, 's, C, DomainRow, 2> {
        DomainRowQuery {
            client,
            params: [user_id, is_admin],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<DomainRowBorrowed, tokio_postgres::Error> {
                    Ok(DomainRowBorrowed {
                        id: row.try_get(0)?,
                        domain: row.try_get(1)?,
                        owner_id: row.try_get(2)?,
                        shared: row.try_get(3)?,
                        catch_all: row.try_get(4)?,
                        random_prefix: row.try_get(5)?,
                        reply_prefix: row.try_get(6)?,
                        default_mailbox_id: row.try_get(7)?,
                        dkim_records: row.try_get(8)?,
                        dns_status: row.try_get(9)?,
                        dns_checked_at: row.try_get(10)?,
                        dns_verified_at: row.try_get(11)?,
                        nb_alias: row.try_get(12)?,
                    })
                },
            mapper: |it| DomainRow::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        ListForUserParams,
        DomainRowQuery<'c, 'a, 's, C, DomainRow, 2>,
        C,
    > for ListForUserStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a ListForUserParams,
    ) -> DomainRowQuery<'c, 'a, 's, C, DomainRow, 2> {
        self.bind(client, &params.user_id, &params.is_admin)
    }
}
pub struct ByIdStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn by_id() -> ByIdStmt {
    ByIdStmt(
        "SELECT d.id, d.domain::text AS domain, d.owner_id, d.shared, d.catch_all, d.random_prefix, d.reply_prefix, d.default_mailbox_id, d.dkim_records, d.dns_status, d.dns_checked_at, d.dns_verified_at, (SELECT COUNT(*) FROM alias a WHERE a.domain_id = d.id) AS nb_alias FROM alias_domain d WHERE d.id = $1",
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
        domain_id: &'a i64,
    ) -> DomainRowQuery<'c, 'a, 's, C, DomainRow, 1> {
        DomainRowQuery {
            client,
            params: [domain_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<DomainRowBorrowed, tokio_postgres::Error> {
                    Ok(DomainRowBorrowed {
                        id: row.try_get(0)?,
                        domain: row.try_get(1)?,
                        owner_id: row.try_get(2)?,
                        shared: row.try_get(3)?,
                        catch_all: row.try_get(4)?,
                        random_prefix: row.try_get(5)?,
                        reply_prefix: row.try_get(6)?,
                        default_mailbox_id: row.try_get(7)?,
                        dkim_records: row.try_get(8)?,
                        dns_status: row.try_get(9)?,
                        dns_checked_at: row.try_get(10)?,
                        dns_verified_at: row.try_get(11)?,
                        nb_alias: row.try_get(12)?,
                    })
                },
            mapper: |it| DomainRow::from(it),
        }
    }
}
pub struct ByIdForUserStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn by_id_for_user() -> ByIdForUserStmt {
    ByIdForUserStmt(
        "SELECT d.id, d.domain::text AS domain, d.owner_id, d.shared, d.catch_all, d.random_prefix, d.reply_prefix, d.default_mailbox_id, d.dkim_records, d.dns_status, d.dns_checked_at, d.dns_verified_at, (SELECT COUNT(*) FROM alias a WHERE a.domain_id = d.id) AS nb_alias FROM alias_domain d WHERE d.id = $1 AND (d.shared OR d.owner_id = $2 OR $3)",
        None,
    )
}
impl ByIdForUserStmt {
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
        domain_id: &'a i64,
        user_id: &'a i64,
        is_admin: &'a bool,
    ) -> DomainRowQuery<'c, 'a, 's, C, DomainRow, 3> {
        DomainRowQuery {
            client,
            params: [domain_id, user_id, is_admin],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<DomainRowBorrowed, tokio_postgres::Error> {
                    Ok(DomainRowBorrowed {
                        id: row.try_get(0)?,
                        domain: row.try_get(1)?,
                        owner_id: row.try_get(2)?,
                        shared: row.try_get(3)?,
                        catch_all: row.try_get(4)?,
                        random_prefix: row.try_get(5)?,
                        reply_prefix: row.try_get(6)?,
                        default_mailbox_id: row.try_get(7)?,
                        dkim_records: row.try_get(8)?,
                        dns_status: row.try_get(9)?,
                        dns_checked_at: row.try_get(10)?,
                        dns_verified_at: row.try_get(11)?,
                        nb_alias: row.try_get(12)?,
                    })
                },
            mapper: |it| DomainRow::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        ByIdForUserParams,
        DomainRowQuery<'c, 'a, 's, C, DomainRow, 3>,
        C,
    > for ByIdForUserStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a ByIdForUserParams,
    ) -> DomainRowQuery<'c, 'a, 's, C, DomainRow, 3> {
        self.bind(client, &params.domain_id, &params.user_id, &params.is_admin)
    }
}
pub struct ListForDashboardStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn list_for_dashboard() -> ListForDashboardStmt {
    ListForDashboardStmt(
        "SELECT d.id, d.domain::text AS domain, d.shared, d.owner_id, d.random_prefix, d.reply_prefix, d.dkim_records, d.dns_status, d.dns_checked_at, d.dns_verified_at, (SELECT COUNT(*) FROM alias a WHERE a.domain_id = d.id) AS nb_alias FROM alias_domain d WHERE d.shared OR d.owner_id = $1 OR $2 ORDER BY d.shared DESC, d.id",
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
        is_admin: &'a bool,
    ) -> ListForDashboardQuery<'c, 'a, 's, C, ListForDashboard, 2> {
        ListForDashboardQuery {
            client,
            params: [user_id, is_admin],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<ListForDashboardBorrowed, tokio_postgres::Error> {
                Ok(ListForDashboardBorrowed {
                    id: row.try_get(0)?,
                    domain: row.try_get(1)?,
                    shared: row.try_get(2)?,
                    owner_id: row.try_get(3)?,
                    random_prefix: row.try_get(4)?,
                    reply_prefix: row.try_get(5)?,
                    dkim_records: row.try_get(6)?,
                    dns_status: row.try_get(7)?,
                    dns_checked_at: row.try_get(8)?,
                    dns_verified_at: row.try_get(9)?,
                    nb_alias: row.try_get(10)?,
                })
            },
            mapper: |it| ListForDashboard::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        ListForDashboardParams,
        ListForDashboardQuery<'c, 'a, 's, C, ListForDashboard, 2>,
        C,
    > for ListForDashboardStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a ListForDashboardParams,
    ) -> ListForDashboardQuery<'c, 'a, 's, C, ListForDashboard, 2> {
        self.bind(client, &params.user_id, &params.is_admin)
    }
}
pub struct ListAdminStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn list_admin() -> ListAdminStmt {
    ListAdminStmt(
        "SELECT d.id, d.domain::text AS domain, d.shared, u.email::text AS owner_email, (SELECT COUNT(*) FROM alias a WHERE a.domain_id = d.id) AS nb_alias FROM alias_domain d LEFT JOIN \"user\" u ON u.id = d.owner_id ORDER BY d.id",
        None,
    )
}
impl ListAdminStmt {
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
    ) -> ListAdminQuery<'c, 'a, 's, C, ListAdmin, 0> {
        ListAdminQuery {
            client,
            params: [],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<ListAdminBorrowed, tokio_postgres::Error> {
                    Ok(ListAdminBorrowed {
                        id: row.try_get(0)?,
                        domain: row.try_get(1)?,
                        shared: row.try_get(2)?,
                        owner_email: row.try_get(3)?,
                        nb_alias: row.try_get(4)?,
                    })
                },
            mapper: |it| ListAdmin::from(it),
        }
    }
}
pub struct ExistsManagableStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn exists_managable() -> ExistsManagableStmt {
    ExistsManagableStmt(
        "SELECT 1 AS one FROM alias_domain WHERE id = $1 AND (owner_id = $2 OR $3)",
        None,
    )
}
impl ExistsManagableStmt {
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
        domain_id: &'a i64,
        user_id: &'a i64,
        is_admin: &'a bool,
    ) -> I32Query<'c, 'a, 's, C, i32, 3> {
        I32Query {
            client,
            params: [domain_id, user_id, is_admin],
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
        ExistsManagableParams,
        I32Query<'c, 'a, 's, C, i32, 3>,
        C,
    > for ExistsManagableStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a ExistsManagableParams,
    ) -> I32Query<'c, 'a, 's, C, i32, 3> {
        self.bind(client, &params.domain_id, &params.user_id, &params.is_admin)
    }
}
pub struct CatchAllAndCapStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn catch_all_and_cap() -> CatchAllAndCapStmt {
    CatchAllAndCapStmt(
        "SELECT catch_all, max_auto_created FROM alias_domain WHERE id = $1",
        None,
    )
}
impl CatchAllAndCapStmt {
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
        domain_id: &'a i64,
    ) -> CatchAllAndCapQuery<'c, 'a, 's, C, CatchAllAndCap, 1> {
        CatchAllAndCapQuery {
            client,
            params: [domain_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<CatchAllAndCap, tokio_postgres::Error> {
                    Ok(CatchAllAndCap {
                        catch_all: row.try_get(0)?,
                        max_auto_created: row.try_get(1)?,
                    })
                },
            mapper: |it| CatchAllAndCap::from(it),
        }
    }
}
pub struct SetCatchAllAndCapStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn set_catch_all_and_cap() -> SetCatchAllAndCapStmt {
    SetCatchAllAndCapStmt(
        "UPDATE alias_domain SET catch_all = $1, max_auto_created = $2 WHERE id = $3",
        None,
    )
}
impl SetCatchAllAndCapStmt {
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
        catch_all: &'a bool,
        max_auto_created: &'a Option<i32>,
        domain_id: &'a i64,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(self.0, &[catch_all, max_auto_created, domain_id])
            .await
    }
}
impl<'a, C: GenericClient + Send + Sync>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        SetCatchAllAndCapParams,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for SetCatchAllAndCapStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a SetCatchAllAndCapParams,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.catch_all,
            &params.max_auto_created,
            &params.domain_id,
        ))
    }
}
pub struct SetRandomPrefixStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn set_random_prefix() -> SetRandomPrefixStmt {
    SetRandomPrefixStmt(
        "UPDATE alias_domain SET random_prefix = $1 WHERE id = $2",
        None,
    )
}
impl SetRandomPrefixStmt {
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
        random_prefix: &'a T1,
        domain_id: &'a i64,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[random_prefix, domain_id]).await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::StringSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        SetRandomPrefixParams<T1>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for SetRandomPrefixStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a SetRandomPrefixParams<T1>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.random_prefix, &params.domain_id))
    }
}
pub struct SetDefaultMailboxStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn set_default_mailbox() -> SetDefaultMailboxStmt {
    SetDefaultMailboxStmt(
        "UPDATE alias_domain SET default_mailbox_id = $1 WHERE id = $2",
        None,
    )
}
impl SetDefaultMailboxStmt {
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
        default_mailbox_id: &'a Option<i64>,
        domain_id: &'a i64,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(self.0, &[default_mailbox_id, domain_id])
            .await
    }
}
impl<'a, C: GenericClient + Send + Sync>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        SetDefaultMailboxParams,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for SetDefaultMailboxStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a SetDefaultMailboxParams,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.default_mailbox_id, &params.domain_id))
    }
}
pub struct SetDkimRecordsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn set_dkim_records() -> SetDkimRecordsStmt {
    SetDkimRecordsStmt(
        "UPDATE alias_domain SET dkim_records = $1 WHERE id = $2",
        None,
    )
}
impl SetDkimRecordsStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient, T1: crate::JsonSql>(
        &'s self,
        client: &'c C,
        dkim_records: &'a T1,
        domain_id: &'a i64,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[dkim_records, domain_id]).await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::JsonSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        SetDkimRecordsParams<T1>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for SetDkimRecordsStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a SetDkimRecordsParams<T1>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.dkim_records, &params.domain_id))
    }
}
pub struct SetDnsCheckStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn set_dns_check() -> SetDnsCheckStmt {
    SetDnsCheckStmt(
        "UPDATE alias_domain SET dns_status = $1, dns_checked_at = $2, dns_verified_at = CASE WHEN $3 THEN COALESCE(dns_verified_at, $2) ELSE dns_verified_at END WHERE id = $4",
        None,
    )
}
impl SetDnsCheckStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient, T1: crate::JsonSql>(
        &'s self,
        client: &'c C,
        dns_status: &'a T1,
        checked_at: &'a time::OffsetDateTime,
        all_verified: &'a bool,
        domain_id: &'a i64,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(self.0, &[dns_status, checked_at, all_verified, domain_id])
            .await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::JsonSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        SetDnsCheckParams<T1>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for SetDnsCheckStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a SetDnsCheckParams<T1>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.dns_status,
            &params.checked_at,
            &params.all_verified,
            &params.domain_id,
        ))
    }
}
pub struct SetSharedStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn set_shared() -> SetSharedStmt {
    SetSharedStmt("UPDATE alias_domain SET shared = $1 WHERE id = $2", None)
}
impl SetSharedStmt {
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
        shared: &'a bool,
        domain_id: &'a i64,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[shared, domain_id]).await
    }
}
impl<'a, C: GenericClient + Send + Sync>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        SetSharedParams,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for SetSharedStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a SetSharedParams,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.shared, &params.domain_id))
    }
}
pub struct DeleteStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn delete() -> DeleteStmt {
    DeleteStmt(
        "DELETE FROM alias_domain WHERE id = $1 AND (owner_id = $2 OR $3)",
        None,
    )
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
        domain_id: &'a i64,
        user_id: &'a i64,
        is_admin: &'a bool,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(self.0, &[domain_id, user_id, is_admin])
            .await
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
        Box::pin(self.bind(client, &params.domain_id, &params.user_id, &params.is_admin))
    }
}
pub struct CreateStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn create() -> CreateStmt {
    CreateStmt(
        "INSERT INTO alias_domain (domain, owner_id, shared, random_prefix) VALUES ($1, $2, FALSE, COALESCE($3, '')) RETURNING id",
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
        domain: &'a T1,
        owner_id: &'a Option<i64>,
        random_prefix: &'a Option<T2>,
    ) -> I64Query<'c, 'a, 's, C, i64, 3> {
        I64Query {
            client,
            params: [domain, owner_id, random_prefix],
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
        I64Query<'c, 'a, 's, C, i64, 3>,
        C,
    > for CreateStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a CreateParams<T1, T2>,
    ) -> I64Query<'c, 'a, 's, C, i64, 3> {
        self.bind(
            client,
            &params.domain,
            &params.owner_id,
            &params.random_prefix,
        )
    }
}
pub struct ByDomainForUserStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn by_domain_for_user() -> ByDomainForUserStmt {
    ByDomainForUserStmt(
        "SELECT id, domain::text AS domain, owner_id, shared, catch_all, random_prefix, reply_prefix, default_mailbox_id, created_at, updated_at FROM alias_domain WHERE domain = $1 AND (shared OR owner_id = $2 OR $3)",
        None,
    )
}
impl ByDomainForUserStmt {
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
        domain: &'a T1,
        user_id: &'a i64,
        is_admin: &'a bool,
    ) -> AliasDomainRowQuery<'c, 'a, 's, C, AliasDomainRow, 3> {
        AliasDomainRowQuery {
            client,
            params: [domain, user_id, is_admin],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<AliasDomainRowBorrowed, tokio_postgres::Error> {
                Ok(AliasDomainRowBorrowed {
                    id: row.try_get(0)?,
                    domain: row.try_get(1)?,
                    owner_id: row.try_get(2)?,
                    shared: row.try_get(3)?,
                    catch_all: row.try_get(4)?,
                    random_prefix: row.try_get(5)?,
                    reply_prefix: row.try_get(6)?,
                    default_mailbox_id: row.try_get(7)?,
                    created_at: row.try_get(8)?,
                    updated_at: row.try_get(9)?,
                })
            },
            mapper: |it| AliasDomainRow::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        ByDomainForUserParams<T1>,
        AliasDomainRowQuery<'c, 'a, 's, C, AliasDomainRow, 3>,
        C,
    > for ByDomainForUserStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a ByDomainForUserParams<T1>,
    ) -> AliasDomainRowQuery<'c, 'a, 's, C, AliasDomainRow, 3> {
        self.bind(client, &params.domain, &params.user_id, &params.is_admin)
    }
}
pub struct FirstAccessibleForUserStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn first_accessible_for_user() -> FirstAccessibleForUserStmt {
    FirstAccessibleForUserStmt(
        "SELECT id, domain::text AS domain, owner_id, shared, catch_all, random_prefix, reply_prefix, default_mailbox_id, created_at, updated_at FROM alias_domain WHERE shared OR owner_id = $1 OR $2 ORDER BY shared DESC, id LIMIT 1",
        None,
    )
}
impl FirstAccessibleForUserStmt {
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
        is_admin: &'a bool,
    ) -> AliasDomainRowQuery<'c, 'a, 's, C, AliasDomainRow, 2> {
        AliasDomainRowQuery {
            client,
            params: [user_id, is_admin],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<AliasDomainRowBorrowed, tokio_postgres::Error> {
                Ok(AliasDomainRowBorrowed {
                    id: row.try_get(0)?,
                    domain: row.try_get(1)?,
                    owner_id: row.try_get(2)?,
                    shared: row.try_get(3)?,
                    catch_all: row.try_get(4)?,
                    random_prefix: row.try_get(5)?,
                    reply_prefix: row.try_get(6)?,
                    default_mailbox_id: row.try_get(7)?,
                    created_at: row.try_get(8)?,
                    updated_at: row.try_get(9)?,
                })
            },
            mapper: |it| AliasDomainRow::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        FirstAccessibleForUserParams,
        AliasDomainRowQuery<'c, 'a, 's, C, AliasDomainRow, 2>,
        C,
    > for FirstAccessibleForUserStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a FirstAccessibleForUserParams,
    ) -> AliasDomainRowQuery<'c, 'a, 's, C, AliasDomainRow, 2> {
        self.bind(client, &params.user_id, &params.is_admin)
    }
}
pub struct IdByDomainStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn id_by_domain() -> IdByDomainStmt {
    IdByDomainStmt("SELECT id FROM alias_domain WHERE domain = $1", None)
}
impl IdByDomainStmt {
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
        domain: &'a T1,
    ) -> I64Query<'c, 'a, 's, C, i64, 1> {
        I64Query {
            client,
            params: [domain],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it,
        }
    }
}
pub struct AllDomainNamesStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn all_domain_names() -> AllDomainNamesStmt {
    AllDomainNamesStmt(
        "SELECT domain::text AS domain FROM alias_domain ORDER BY domain",
        None,
    )
}
impl AllDomainNamesStmt {
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
    ) -> StringQuery<'c, 'a, 's, C, String, 0> {
        StringQuery {
            client,
            params: [],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it.into(),
        }
    }
}
pub struct SetDefaultMailboxByOwnerEmailStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn set_default_mailbox_by_owner_email() -> SetDefaultMailboxByOwnerEmailStmt {
    SetDefaultMailboxByOwnerEmailStmt(
        "UPDATE alias_domain d SET default_mailbox_id = ( SELECT m.id FROM mailbox m WHERE m.email = $1 AND m.user_id = d.owner_id AND m.enabled AND m.verified ) WHERE d.domain = $2",
        None,
    )
}
impl SetDefaultMailboxByOwnerEmailStmt {
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
        mailbox_email: &'a T1,
        domain: &'a T2,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[mailbox_email, domain]).await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::StringSql, T2: crate::StringSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        SetDefaultMailboxByOwnerEmailParams<T1, T2>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for SetDefaultMailboxByOwnerEmailStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a SetDefaultMailboxByOwnerEmailParams<T1, T2>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.mailbox_email, &params.domain))
    }
}
pub struct DefaultMailboxIsNullStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn default_mailbox_is_null() -> DefaultMailboxIsNullStmt {
    DefaultMailboxIsNullStmt(
        "SELECT default_mailbox_id IS NULL AS is_null FROM alias_domain WHERE domain = $1",
        None,
    )
}
impl DefaultMailboxIsNullStmt {
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
        domain: &'a T1,
    ) -> OptionboolQuery<'c, 'a, 's, C, Option<bool>, 1> {
        OptionboolQuery {
            client,
            params: [domain],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it,
        }
    }
}
