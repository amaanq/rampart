// This file was generated with `cornucopia`. Do not modify.

#[derive(Clone, Copy, Debug)]
pub struct EmailExistsForOtherParams<T1: crate::StringSql> {
    pub email: T1,
    pub exclude_id: i64,
}
#[derive(Debug)]
pub struct CreateParams<T1: crate::StringSql, T2: crate::StringSql, T3: crate::StringSql> {
    pub email: T1,
    pub password_hash: Option<T2>,
    pub display_name: Option<T3>,
    pub is_admin: bool,
}
#[derive(Debug)]
pub struct CreateFirstAdminParams<T1: crate::StringSql, T2: crate::StringSql, T3: crate::StringSql>
{
    pub email: T1,
    pub password_hash: T2,
    pub display_name: Option<T3>,
}
#[derive(Debug)]
pub struct CreateViaInviteParams<T1: crate::StringSql, T2: crate::StringSql, T3: crate::StringSql> {
    pub email: T1,
    pub password_hash: T2,
    pub display_name: Option<T3>,
}
#[derive(Debug)]
pub struct SetPasswordParams<T1: crate::StringSql> {
    pub password_hash: Option<T1>,
    pub user_id: i64,
}
#[derive(Clone, Copy, Debug)]
pub struct SetEmailParams<T1: crate::StringSql> {
    pub email: T1,
    pub user_id: i64,
}
#[derive(Clone, Copy, Debug)]
pub struct SetAdminParams {
    pub is_admin: bool,
    pub user_id: i64,
}
#[derive(Clone, Copy, Debug)]
pub struct CapAndCountAliasesParams {
    pub default_cap: i64,
    pub user_id: i64,
}
#[derive(Clone, Copy, Debug)]
pub struct CapAndCountDomainsParams {
    pub default_cap: i64,
    pub user_id: i64,
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ByEmailForBasicAuth {
    pub id: i64,
    pub email: String,
    pub password_hash: Option<String>,
    pub enabled: bool,
    pub is_admin: bool,
    pub display_name: Option<String>,
    pub max_aliases: Option<i32>,
    pub max_domains: Option<i32>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: time::OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: time::OffsetDateTime,
}
pub struct ByEmailForBasicAuthBorrowed<'a> {
    pub id: i64,
    pub email: &'a str,
    pub password_hash: Option<&'a str>,
    pub enabled: bool,
    pub is_admin: bool,
    pub display_name: Option<&'a str>,
    pub max_aliases: Option<i32>,
    pub max_domains: Option<i32>,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}
impl<'a> From<ByEmailForBasicAuthBorrowed<'a>> for ByEmailForBasicAuth {
    fn from(
        ByEmailForBasicAuthBorrowed {
            id,
            email,
            password_hash,
            enabled,
            is_admin,
            display_name,
            max_aliases,
            max_domains,
            created_at,
            updated_at,
        }: ByEmailForBasicAuthBorrowed<'a>,
    ) -> Self {
        Self {
            id,
            email: email.into(),
            password_hash: password_hash.map(|v| v.into()),
            enabled,
            is_admin,
            display_name: display_name.map(|v| v.into()),
            max_aliases,
            max_domains,
            created_at,
            updated_at,
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Info {
    pub email: String,
    pub is_admin: bool,
    pub alias_count: i64,
    pub mailbox_count: i64,
    pub domain_count: i64,
}
pub struct InfoBorrowed<'a> {
    pub email: &'a str,
    pub is_admin: bool,
    pub alias_count: i64,
    pub mailbox_count: i64,
    pub domain_count: i64,
}
impl<'a> From<InfoBorrowed<'a>> for Info {
    fn from(
        InfoBorrowed {
            email,
            is_admin,
            alias_count,
            mailbox_count,
            domain_count,
        }: InfoBorrowed<'a>,
    ) -> Self {
        Self {
            email: email.into(),
            is_admin,
            alias_count,
            mailbox_count,
            domain_count,
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct DisplayForWebauthn {
    pub email: String,
    pub display_name: Option<String>,
}
pub struct DisplayForWebauthnBorrowed<'a> {
    pub email: &'a str,
    pub display_name: Option<&'a str>,
}
impl<'a> From<DisplayForWebauthnBorrowed<'a>> for DisplayForWebauthn {
    fn from(
        DisplayForWebauthnBorrowed {
            email,
            display_name,
        }: DisplayForWebauthnBorrowed<'a>,
    ) -> Self {
        Self {
            email: email.into(),
            display_name: display_name.map(|v| v.into()),
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ListAdmin {
    pub id: i64,
    pub email: String,
    pub enabled: bool,
    pub is_admin: bool,
    pub display_name: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: time::OffsetDateTime,
    pub nb_aliases: i64,
    pub nb_mailboxes: i64,
    pub nb_domains: i64,
}
pub struct ListAdminBorrowed<'a> {
    pub id: i64,
    pub email: &'a str,
    pub enabled: bool,
    pub is_admin: bool,
    pub display_name: Option<&'a str>,
    pub created_at: time::OffsetDateTime,
    pub nb_aliases: i64,
    pub nb_mailboxes: i64,
    pub nb_domains: i64,
}
impl<'a> From<ListAdminBorrowed<'a>> for ListAdmin {
    fn from(
        ListAdminBorrowed {
            id,
            email,
            enabled,
            is_admin,
            display_name,
            created_at,
            nb_aliases,
            nb_mailboxes,
            nb_domains,
        }: ListAdminBorrowed<'a>,
    ) -> Self {
        Self {
            id,
            email: email.into(),
            enabled,
            is_admin,
            display_name: display_name.map(|v| v.into()),
            created_at,
            nb_aliases,
            nb_mailboxes,
            nb_domains,
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ListAdminCompact {
    pub id: i64,
    pub email: String,
    pub enabled: bool,
    pub is_admin: bool,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: time::OffsetDateTime,
    pub nb_aliases: i64,
    pub nb_domains: i64,
}
pub struct ListAdminCompactBorrowed<'a> {
    pub id: i64,
    pub email: &'a str,
    pub enabled: bool,
    pub is_admin: bool,
    pub created_at: time::OffsetDateTime,
    pub nb_aliases: i64,
    pub nb_domains: i64,
}
impl<'a> From<ListAdminCompactBorrowed<'a>> for ListAdminCompact {
    fn from(
        ListAdminCompactBorrowed {
            id,
            email,
            enabled,
            is_admin,
            created_at,
            nb_aliases,
            nb_domains,
        }: ListAdminCompactBorrowed<'a>,
    ) -> Self {
        Self {
            id,
            email: email.into(),
            enabled,
            is_admin,
            created_at,
            nb_aliases,
            nb_domains,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Copy, serde::Serialize)]
pub struct CreateViaInvite {
    pub id: i64,
    pub is_admin: bool,
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ListCli {
    pub id: i64,
    pub email: String,
    pub is_admin: bool,
    pub enabled: bool,
    pub display_name: Option<String>,
}
pub struct ListCliBorrowed<'a> {
    pub id: i64,
    pub email: &'a str,
    pub is_admin: bool,
    pub enabled: bool,
    pub display_name: Option<&'a str>,
}
impl<'a> From<ListCliBorrowed<'a>> for ListCli {
    fn from(
        ListCliBorrowed {
            id,
            email,
            is_admin,
            enabled,
            display_name,
        }: ListCliBorrowed<'a>,
    ) -> Self {
        Self {
            id,
            email: email.into(),
            is_admin,
            enabled,
            display_name: display_name.map(|v| v.into()),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Copy, serde::Serialize)]
pub struct CapAndCountAliases {
    pub cap: i64,
    pub current: i64,
}
#[derive(Debug, Clone, PartialEq, Copy, serde::Serialize)]
pub struct CapAndCountDomains {
    pub cap: i64,
    pub current: i64,
}
use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
pub struct OptionStringQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<Option<&str>, tokio_postgres::Error>,
    mapper: fn(Option<&str>) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> OptionStringQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(self, mapper: fn(Option<&str>) -> R) -> OptionStringQuery<'c, 'a, 's, C, R, N> {
        OptionStringQuery {
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
pub struct ByEmailForBasicAuthQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor:
        fn(&tokio_postgres::Row) -> Result<ByEmailForBasicAuthBorrowed, tokio_postgres::Error>,
    mapper: fn(ByEmailForBasicAuthBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> ByEmailForBasicAuthQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(ByEmailForBasicAuthBorrowed) -> R,
    ) -> ByEmailForBasicAuthQuery<'c, 'a, 's, C, R, N> {
        ByEmailForBasicAuthQuery {
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
pub struct InfoQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<InfoBorrowed, tokio_postgres::Error>,
    mapper: fn(InfoBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> InfoQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(self, mapper: fn(InfoBorrowed) -> R) -> InfoQuery<'c, 'a, 's, C, R, N> {
        InfoQuery {
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
pub struct DisplayForWebauthnQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor:
        fn(&tokio_postgres::Row) -> Result<DisplayForWebauthnBorrowed, tokio_postgres::Error>,
    mapper: fn(DisplayForWebauthnBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> DisplayForWebauthnQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(DisplayForWebauthnBorrowed) -> R,
    ) -> DisplayForWebauthnQuery<'c, 'a, 's, C, R, N> {
        DisplayForWebauthnQuery {
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
pub struct ListAdminCompactQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<ListAdminCompactBorrowed, tokio_postgres::Error>,
    mapper: fn(ListAdminCompactBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> ListAdminCompactQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(ListAdminCompactBorrowed) -> R,
    ) -> ListAdminCompactQuery<'c, 'a, 's, C, R, N> {
        ListAdminCompactQuery {
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
pub struct BoolQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<bool, tokio_postgres::Error>,
    mapper: fn(bool) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> BoolQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(self, mapper: fn(bool) -> R) -> BoolQuery<'c, 'a, 's, C, R, N> {
        BoolQuery {
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
pub struct CreateViaInviteQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<CreateViaInvite, tokio_postgres::Error>,
    mapper: fn(CreateViaInvite) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> CreateViaInviteQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(CreateViaInvite) -> R,
    ) -> CreateViaInviteQuery<'c, 'a, 's, C, R, N> {
        CreateViaInviteQuery {
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
pub struct ListCliQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<ListCliBorrowed, tokio_postgres::Error>,
    mapper: fn(ListCliBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> ListCliQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(self, mapper: fn(ListCliBorrowed) -> R) -> ListCliQuery<'c, 'a, 's, C, R, N> {
        ListCliQuery {
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
pub struct CapAndCountAliasesQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<CapAndCountAliases, tokio_postgres::Error>,
    mapper: fn(CapAndCountAliases) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> CapAndCountAliasesQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(CapAndCountAliases) -> R,
    ) -> CapAndCountAliasesQuery<'c, 'a, 's, C, R, N> {
        CapAndCountAliasesQuery {
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
pub struct CapAndCountDomainsQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<CapAndCountDomains, tokio_postgres::Error>,
    mapper: fn(CapAndCountDomains) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> CapAndCountDomainsQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(CapAndCountDomains) -> R,
    ) -> CapAndCountDomainsQuery<'c, 'a, 's, C, R, N> {
        CapAndCountDomainsQuery {
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
pub struct ByIdWithPwhashStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn by_id_with_pwhash() -> ByIdWithPwhashStmt {
    ByIdWithPwhashStmt("SELECT password_hash FROM \"user\" WHERE id = $1", None)
}
impl ByIdWithPwhashStmt {
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
    ) -> OptionStringQuery<'c, 'a, 's, C, Option<String>, 1> {
        OptionStringQuery {
            client,
            params: [user_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it.map(|v| v.into()),
        }
    }
}
pub struct ByEmailForBasicAuthStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn by_email_for_basic_auth() -> ByEmailForBasicAuthStmt {
    ByEmailForBasicAuthStmt(
        "SELECT id, email::text AS email, password_hash, enabled, is_admin, display_name, max_aliases, max_domains, created_at, updated_at FROM \"user\" WHERE email = $1 AND enabled",
        None,
    )
}
impl ByEmailForBasicAuthStmt {
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
        email: &'a T1,
    ) -> ByEmailForBasicAuthQuery<'c, 'a, 's, C, ByEmailForBasicAuth, 1> {
        ByEmailForBasicAuthQuery {
            client,
            params: [email],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<ByEmailForBasicAuthBorrowed, tokio_postgres::Error> {
                Ok(ByEmailForBasicAuthBorrowed {
                    id: row.try_get(0)?,
                    email: row.try_get(1)?,
                    password_hash: row.try_get(2)?,
                    enabled: row.try_get(3)?,
                    is_admin: row.try_get(4)?,
                    display_name: row.try_get(5)?,
                    max_aliases: row.try_get(6)?,
                    max_domains: row.try_get(7)?,
                    created_at: row.try_get(8)?,
                    updated_at: row.try_get(9)?,
                })
            },
            mapper: |it| ByEmailForBasicAuth::from(it),
        }
    }
}
pub struct ByEmailIdStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn by_email_id() -> ByEmailIdStmt {
    ByEmailIdStmt("SELECT id FROM \"user\" WHERE email = $1 AND enabled", None)
}
impl ByEmailIdStmt {
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
        email: &'a T1,
    ) -> I64Query<'c, 'a, 's, C, i64, 1> {
        I64Query {
            client,
            params: [email],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it,
        }
    }
}
pub struct ByEmailIdUnfilteredStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn by_email_id_unfiltered() -> ByEmailIdUnfilteredStmt {
    ByEmailIdUnfilteredStmt("SELECT id FROM \"user\" WHERE email = $1", None)
}
impl ByEmailIdUnfilteredStmt {
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
        email: &'a T1,
    ) -> I64Query<'c, 'a, 's, C, i64, 1> {
        I64Query {
            client,
            params: [email],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it,
        }
    }
}
pub struct EmailExistsForOtherStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn email_exists_for_other() -> EmailExistsForOtherStmt {
    EmailExistsForOtherStmt(
        "SELECT 1 AS one FROM \"user\" WHERE email = $1 AND id <> $2",
        None,
    )
}
impl EmailExistsForOtherStmt {
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
        email: &'a T1,
        exclude_id: &'a i64,
    ) -> I32Query<'c, 'a, 's, C, i32, 2> {
        I32Query {
            client,
            params: [email, exclude_id],
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
        EmailExistsForOtherParams<T1>,
        I32Query<'c, 'a, 's, C, i32, 2>,
        C,
    > for EmailExistsForOtherStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a EmailExistsForOtherParams<T1>,
    ) -> I32Query<'c, 'a, 's, C, i32, 2> {
        self.bind(client, &params.email, &params.exclude_id)
    }
}
pub struct InfoStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn info() -> InfoStmt {
    InfoStmt(
        "SELECT u.email::text AS email, u.is_admin, (SELECT COUNT(*) FROM alias WHERE user_id = u.id)         AS alias_count, (SELECT COUNT(*) FROM mailbox WHERE user_id = u.id)       AS mailbox_count, (SELECT COUNT(*) FROM alias_domain WHERE owner_id = u.id) AS domain_count FROM \"user\" u WHERE u.id = $1",
        None,
    )
}
impl InfoStmt {
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
    ) -> InfoQuery<'c, 'a, 's, C, Info, 1> {
        InfoQuery {
            client,
            params: [user_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row: &tokio_postgres::Row| -> Result<InfoBorrowed, tokio_postgres::Error> {
                Ok(InfoBorrowed {
                    email: row.try_get(0)?,
                    is_admin: row.try_get(1)?,
                    alias_count: row.try_get(2)?,
                    mailbox_count: row.try_get(3)?,
                    domain_count: row.try_get(4)?,
                })
            },
            mapper: |it| Info::from(it),
        }
    }
}
pub struct EmailByIdStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn email_by_id() -> EmailByIdStmt {
    EmailByIdStmt(
        "SELECT email::text AS email FROM \"user\" WHERE id = $1",
        None,
    )
}
impl EmailByIdStmt {
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
    ) -> StringQuery<'c, 'a, 's, C, String, 1> {
        StringQuery {
            client,
            params: [user_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it.into(),
        }
    }
}
pub struct DisplayForWebauthnStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn display_for_webauthn() -> DisplayForWebauthnStmt {
    DisplayForWebauthnStmt(
        "SELECT email::text AS email, display_name FROM \"user\" WHERE id = $1",
        None,
    )
}
impl DisplayForWebauthnStmt {
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
    ) -> DisplayForWebauthnQuery<'c, 'a, 's, C, DisplayForWebauthn, 1> {
        DisplayForWebauthnQuery {
            client,
            params: [user_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<DisplayForWebauthnBorrowed, tokio_postgres::Error> {
                Ok(DisplayForWebauthnBorrowed {
                    email: row.try_get(0)?,
                    display_name: row.try_get(1)?,
                })
            },
            mapper: |it| DisplayForWebauthn::from(it),
        }
    }
}
pub struct ListAdminStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn list_admin() -> ListAdminStmt {
    ListAdminStmt(
        "SELECT u.id, u.email::text AS email, u.enabled, u.is_admin, u.display_name, u.created_at, (SELECT COUNT(*) FROM alias WHERE user_id = u.id)         AS nb_aliases, (SELECT COUNT(*) FROM mailbox WHERE user_id = u.id)       AS nb_mailboxes, (SELECT COUNT(*) FROM alias_domain WHERE owner_id = u.id) AS nb_domains FROM \"user\" u ORDER BY u.id",
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
                        email: row.try_get(1)?,
                        enabled: row.try_get(2)?,
                        is_admin: row.try_get(3)?,
                        display_name: row.try_get(4)?,
                        created_at: row.try_get(5)?,
                        nb_aliases: row.try_get(6)?,
                        nb_mailboxes: row.try_get(7)?,
                        nb_domains: row.try_get(8)?,
                    })
                },
            mapper: |it| ListAdmin::from(it),
        }
    }
}
pub struct ListAdminCompactStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn list_admin_compact() -> ListAdminCompactStmt {
    ListAdminCompactStmt(
        "SELECT u.id, u.email::text AS email, u.enabled, u.is_admin, u.created_at, (SELECT COUNT(*) FROM alias WHERE user_id = u.id)         AS nb_aliases, (SELECT COUNT(*) FROM alias_domain WHERE owner_id = u.id) AS nb_domains FROM \"user\" u ORDER BY u.id",
        None,
    )
}
impl ListAdminCompactStmt {
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
    ) -> ListAdminCompactQuery<'c, 'a, 's, C, ListAdminCompact, 0> {
        ListAdminCompactQuery {
            client,
            params: [],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<ListAdminCompactBorrowed, tokio_postgres::Error> {
                Ok(ListAdminCompactBorrowed {
                    id: row.try_get(0)?,
                    email: row.try_get(1)?,
                    enabled: row.try_get(2)?,
                    is_admin: row.try_get(3)?,
                    created_at: row.try_get(4)?,
                    nb_aliases: row.try_get(5)?,
                    nb_domains: row.try_get(6)?,
                })
            },
            mapper: |it| ListAdminCompact::from(it),
        }
    }
}
pub struct CreateStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn create() -> CreateStmt {
    CreateStmt(
        "INSERT INTO \"user\" (email, password_hash, display_name, is_admin) VALUES ($1, $2, $3, $4) RETURNING id",
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
        email: &'a T1,
        password_hash: &'a Option<T2>,
        display_name: &'a Option<T3>,
        is_admin: &'a bool,
    ) -> I64Query<'c, 'a, 's, C, i64, 4> {
        I64Query {
            client,
            params: [email, password_hash, display_name, is_admin],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it,
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::StringSql, T2: crate::StringSql, T3: crate::StringSql>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        CreateParams<T1, T2, T3>,
        I64Query<'c, 'a, 's, C, i64, 4>,
        C,
    > for CreateStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a CreateParams<T1, T2, T3>,
    ) -> I64Query<'c, 'a, 's, C, i64, 4> {
        self.bind(
            client,
            &params.email,
            &params.password_hash,
            &params.display_name,
            &params.is_admin,
        )
    }
}
pub struct AnyExistsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn any_exists() -> AnyExistsStmt {
    AnyExistsStmt("SELECT EXISTS(SELECT 1 FROM \"user\") AS exists", None)
}
impl AnyExistsStmt {
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
    ) -> BoolQuery<'c, 'a, 's, C, bool, 0> {
        BoolQuery {
            client,
            params: [],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it,
        }
    }
}
pub struct CreateFirstAdminStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn create_first_admin() -> CreateFirstAdminStmt {
    CreateFirstAdminStmt(
        "INSERT INTO \"user\" (email, password_hash, display_name, is_admin) SELECT $1, $2, $3, TRUE WHERE NOT EXISTS (SELECT 1 FROM \"user\") RETURNING id",
        None,
    )
}
impl CreateFirstAdminStmt {
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
        email: &'a T1,
        password_hash: &'a T2,
        display_name: &'a Option<T3>,
    ) -> I64Query<'c, 'a, 's, C, i64, 3> {
        I64Query {
            client,
            params: [email, password_hash, display_name],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it,
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::StringSql, T2: crate::StringSql, T3: crate::StringSql>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        CreateFirstAdminParams<T1, T2, T3>,
        I64Query<'c, 'a, 's, C, i64, 3>,
        C,
    > for CreateFirstAdminStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a CreateFirstAdminParams<T1, T2, T3>,
    ) -> I64Query<'c, 'a, 's, C, i64, 3> {
        self.bind(
            client,
            &params.email,
            &params.password_hash,
            &params.display_name,
        )
    }
}
pub struct CreateViaInviteStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn create_via_invite() -> CreateViaInviteStmt {
    CreateViaInviteStmt(
        "INSERT INTO \"user\" (email, password_hash, display_name) VALUES ($1, $2, $3) RETURNING id, is_admin",
        None,
    )
}
impl CreateViaInviteStmt {
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
        email: &'a T1,
        password_hash: &'a T2,
        display_name: &'a Option<T3>,
    ) -> CreateViaInviteQuery<'c, 'a, 's, C, CreateViaInvite, 3> {
        CreateViaInviteQuery {
            client,
            params: [email, password_hash, display_name],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<CreateViaInvite, tokio_postgres::Error> {
                    Ok(CreateViaInvite {
                        id: row.try_get(0)?,
                        is_admin: row.try_get(1)?,
                    })
                },
            mapper: |it| CreateViaInvite::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::StringSql, T2: crate::StringSql, T3: crate::StringSql>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        CreateViaInviteParams<T1, T2, T3>,
        CreateViaInviteQuery<'c, 'a, 's, C, CreateViaInvite, 3>,
        C,
    > for CreateViaInviteStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a CreateViaInviteParams<T1, T2, T3>,
    ) -> CreateViaInviteQuery<'c, 'a, 's, C, CreateViaInvite, 3> {
        self.bind(
            client,
            &params.email,
            &params.password_hash,
            &params.display_name,
        )
    }
}
pub struct SetPasswordStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn set_password() -> SetPasswordStmt {
    SetPasswordStmt("UPDATE \"user\" SET password_hash = $1 WHERE id = $2", None)
}
impl SetPasswordStmt {
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
        password_hash: &'a Option<T1>,
        user_id: &'a i64,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[password_hash, user_id]).await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::StringSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        SetPasswordParams<T1>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for SetPasswordStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a SetPasswordParams<T1>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.password_hash, &params.user_id))
    }
}
pub struct SetEmailStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn set_email() -> SetEmailStmt {
    SetEmailStmt("UPDATE \"user\" SET email = $1 WHERE id = $2", None)
}
impl SetEmailStmt {
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
        email: &'a T1,
        user_id: &'a i64,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[email, user_id]).await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::StringSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        SetEmailParams<T1>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for SetEmailStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a SetEmailParams<T1>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.email, &params.user_id))
    }
}
pub struct EnableStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn enable() -> EnableStmt {
    EnableStmt("UPDATE \"user\" SET enabled = TRUE WHERE id = $1", None)
}
impl EnableStmt {
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
pub struct DisableStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn disable() -> DisableStmt {
    DisableStmt("UPDATE \"user\" SET enabled = FALSE WHERE id = $1", None)
}
impl DisableStmt {
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
pub struct SetAdminStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn set_admin() -> SetAdminStmt {
    SetAdminStmt("UPDATE \"user\" SET is_admin = $1 WHERE id = $2", None)
}
impl SetAdminStmt {
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
        is_admin: &'a bool,
        user_id: &'a i64,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[is_admin, user_id]).await
    }
}
impl<'a, C: GenericClient + Send + Sync>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        SetAdminParams,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for SetAdminStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a SetAdminParams,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.is_admin, &params.user_id))
    }
}
pub struct ListCliStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn list_cli() -> ListCliStmt {
    ListCliStmt(
        "SELECT id, email::text AS email, is_admin, enabled, display_name FROM \"user\" ORDER BY id",
        None,
    )
}
impl ListCliStmt {
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
    ) -> ListCliQuery<'c, 'a, 's, C, ListCli, 0> {
        ListCliQuery {
            client,
            params: [],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<ListCliBorrowed, tokio_postgres::Error> {
                    Ok(ListCliBorrowed {
                        id: row.try_get(0)?,
                        email: row.try_get(1)?,
                        is_admin: row.try_get(2)?,
                        enabled: row.try_get(3)?,
                        display_name: row.try_get(4)?,
                    })
                },
            mapper: |it| ListCli::from(it),
        }
    }
}
pub struct CapAndCountAliasesStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn cap_and_count_aliases() -> CapAndCountAliasesStmt {
    CapAndCountAliasesStmt(
        "SELECT COALESCE(u.max_aliases::bigint, $1) AS cap, (SELECT COUNT(*) FROM alias WHERE user_id = u.id) AS current FROM \"user\" u WHERE u.id = $2",
        None,
    )
}
impl CapAndCountAliasesStmt {
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
        default_cap: &'a i64,
        user_id: &'a i64,
    ) -> CapAndCountAliasesQuery<'c, 'a, 's, C, CapAndCountAliases, 2> {
        CapAndCountAliasesQuery {
            client,
            params: [default_cap, user_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<CapAndCountAliases, tokio_postgres::Error> {
                    Ok(CapAndCountAliases {
                        cap: row.try_get(0)?,
                        current: row.try_get(1)?,
                    })
                },
            mapper: |it| CapAndCountAliases::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        CapAndCountAliasesParams,
        CapAndCountAliasesQuery<'c, 'a, 's, C, CapAndCountAliases, 2>,
        C,
    > for CapAndCountAliasesStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a CapAndCountAliasesParams,
    ) -> CapAndCountAliasesQuery<'c, 'a, 's, C, CapAndCountAliases, 2> {
        self.bind(client, &params.default_cap, &params.user_id)
    }
}
pub struct CapAndCountDomainsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn cap_and_count_domains() -> CapAndCountDomainsStmt {
    CapAndCountDomainsStmt(
        "SELECT COALESCE(u.max_domains::bigint, $1) AS cap, (SELECT COUNT(*) FROM alias_domain WHERE owner_id = u.id) AS current FROM \"user\" u WHERE u.id = $2",
        None,
    )
}
impl CapAndCountDomainsStmt {
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
        default_cap: &'a i64,
        user_id: &'a i64,
    ) -> CapAndCountDomainsQuery<'c, 'a, 's, C, CapAndCountDomains, 2> {
        CapAndCountDomainsQuery {
            client,
            params: [default_cap, user_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<CapAndCountDomains, tokio_postgres::Error> {
                    Ok(CapAndCountDomains {
                        cap: row.try_get(0)?,
                        current: row.try_get(1)?,
                    })
                },
            mapper: |it| CapAndCountDomains::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        CapAndCountDomainsParams,
        CapAndCountDomainsQuery<'c, 'a, 's, C, CapAndCountDomains, 2>,
        C,
    > for CapAndCountDomainsStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a CapAndCountDomainsParams,
    ) -> CapAndCountDomainsQuery<'c, 'a, 's, C, CapAndCountDomains, 2> {
        self.bind(client, &params.default_cap, &params.user_id)
    }
}
