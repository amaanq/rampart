// This file was generated with `clorinde`. Do not modify.

#[derive(Clone, Copy, Debug)]
pub struct ByIdUserParams {
    pub mailbox_id: i64,
    pub user_id: i64,
}
#[derive(Clone, Copy, Debug)]
pub struct ExistsVerifiedParams {
    pub mailbox_id: i64,
    pub user_id: i64,
}
#[derive(Clone, Copy, Debug)]
pub struct IdIfVerifiedParams {
    pub mailbox_id: i64,
    pub user_id: i64,
}
#[derive(Clone, Copy, Debug)]
pub struct IdForUserEmailParams<T1: crate::StringSql> {
    pub user_id: i64,
    pub email: T1,
}
#[derive(Clone, Copy, Debug)]
pub struct VerifiedForUserParams {
    pub mailbox_id: i64,
    pub user_id: i64,
}
#[derive(Debug)]
pub struct CreateParams<T1: crate::StringSql, T2: crate::StringSql> {
    pub user_id: i64,
    pub email: T1,
    pub display_name: Option<T2>,
}
#[derive(Debug)]
pub struct CreateVerifiedParams<T1: crate::StringSql, T2: crate::StringSql> {
    pub user_id: i64,
    pub email: T1,
    pub display_name: Option<T2>,
}
#[derive(Debug)]
pub struct SetDisplayNameParams<T1: crate::StringSql> {
    pub display_name: Option<T1>,
    pub mailbox_id: i64,
    pub user_id: i64,
}
#[derive(Clone, Copy, Debug)]
pub struct SetEnabledParams {
    pub enabled: bool,
    pub mailbox_id: i64,
    pub user_id: i64,
}
#[derive(Clone, Copy, Debug)]
pub struct DeleteParams {
    pub mailbox_id: i64,
    pub user_id: i64,
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct MailboxRow {
    pub id: i64,
    pub email: String,
    pub display_name: Option<String>,
    pub verified: bool,
    pub enabled: bool,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: time::OffsetDateTime,
    pub nb_alias: i64,
}
pub struct MailboxRowBorrowed<'a> {
    pub id: i64,
    pub email: &'a str,
    pub display_name: Option<&'a str>,
    pub verified: bool,
    pub enabled: bool,
    pub created_at: time::OffsetDateTime,
    pub nb_alias: i64,
}
impl<'a> From<MailboxRowBorrowed<'a>> for MailboxRow {
    fn from(
        MailboxRowBorrowed {
            id,
            email,
            display_name,
            verified,
            enabled,
            created_at,
            nb_alias,
        }: MailboxRowBorrowed<'a>,
    ) -> Self {
        Self {
            id,
            email: email.into(),
            display_name: display_name.map(|v| v.into()),
            verified,
            enabled,
            created_at,
            nb_alias,
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct EmailAndVerified {
    pub email: String,
    pub verified: bool,
}
pub struct EmailAndVerifiedBorrowed<'a> {
    pub email: &'a str,
    pub verified: bool,
}
impl<'a> From<EmailAndVerifiedBorrowed<'a>> for EmailAndVerified {
    fn from(EmailAndVerifiedBorrowed { email, verified }: EmailAndVerifiedBorrowed<'a>) -> Self {
        Self {
            email: email.into(),
            verified,
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct MailboxAdminRow {
    pub id: i64,
    pub user_email: String,
    pub email: String,
    pub display_name: Option<String>,
    pub verified: bool,
    pub enabled: bool,
}
pub struct MailboxAdminRowBorrowed<'a> {
    pub id: i64,
    pub user_email: &'a str,
    pub email: &'a str,
    pub display_name: Option<&'a str>,
    pub verified: bool,
    pub enabled: bool,
}
impl<'a> From<MailboxAdminRowBorrowed<'a>> for MailboxAdminRow {
    fn from(
        MailboxAdminRowBorrowed {
            id,
            user_email,
            email,
            display_name,
            verified,
            enabled,
        }: MailboxAdminRowBorrowed<'a>,
    ) -> Self {
        Self {
            id,
            user_email: user_email.into(),
            email: email.into(),
            display_name: display_name.map(|v| v.into()),
            verified,
            enabled,
        }
    }
}
use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
pub struct MailboxRowQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<MailboxRowBorrowed, tokio_postgres::Error>,
    mapper: fn(MailboxRowBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> MailboxRowQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(MailboxRowBorrowed) -> R,
    ) -> MailboxRowQuery<'c, 'a, 's, C, R, N> {
        MailboxRowQuery {
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
pub struct EmailAndVerifiedQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<EmailAndVerifiedBorrowed, tokio_postgres::Error>,
    mapper: fn(EmailAndVerifiedBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> EmailAndVerifiedQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(EmailAndVerifiedBorrowed) -> R,
    ) -> EmailAndVerifiedQuery<'c, 'a, 's, C, R, N> {
        EmailAndVerifiedQuery {
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
pub struct MailboxAdminRowQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<MailboxAdminRowBorrowed, tokio_postgres::Error>,
    mapper: fn(MailboxAdminRowBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> MailboxAdminRowQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(MailboxAdminRowBorrowed) -> R,
    ) -> MailboxAdminRowQuery<'c, 'a, 's, C, R, N> {
        MailboxAdminRowQuery {
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
        "SELECT m.id, m.email::text AS email, m.display_name, m.verified, m.enabled, m.created_at, (SELECT COUNT(*) FROM alias a WHERE a.mailbox_id = m.id) AS nb_alias FROM mailbox m WHERE m.user_id = $1 ORDER BY m.id",
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
    ) -> MailboxRowQuery<'c, 'a, 's, C, MailboxRow, 1> {
        MailboxRowQuery {
            client,
            params: [user_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<MailboxRowBorrowed, tokio_postgres::Error> {
                    Ok(MailboxRowBorrowed {
                        id: row.try_get(0)?,
                        email: row.try_get(1)?,
                        display_name: row.try_get(2)?,
                        verified: row.try_get(3)?,
                        enabled: row.try_get(4)?,
                        created_at: row.try_get(5)?,
                        nb_alias: row.try_get(6)?,
                    })
                },
            mapper: |it| MailboxRow::from(it),
        }
    }
}
pub struct ByIdStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn by_id() -> ByIdStmt {
    ByIdStmt(
        "SELECT m.id, m.email::text AS email, m.display_name, m.verified, m.enabled, m.created_at, (SELECT COUNT(*) FROM alias a WHERE a.mailbox_id = m.id) AS nb_alias FROM mailbox m WHERE m.id = $1",
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
        mailbox_id: &'a i64,
    ) -> MailboxRowQuery<'c, 'a, 's, C, MailboxRow, 1> {
        MailboxRowQuery {
            client,
            params: [mailbox_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<MailboxRowBorrowed, tokio_postgres::Error> {
                    Ok(MailboxRowBorrowed {
                        id: row.try_get(0)?,
                        email: row.try_get(1)?,
                        display_name: row.try_get(2)?,
                        verified: row.try_get(3)?,
                        enabled: row.try_get(4)?,
                        created_at: row.try_get(5)?,
                        nb_alias: row.try_get(6)?,
                    })
                },
            mapper: |it| MailboxRow::from(it),
        }
    }
}
pub struct ByIdUserStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn by_id_user() -> ByIdUserStmt {
    ByIdUserStmt(
        "SELECT m.id, m.email::text AS email, m.display_name, m.verified, m.enabled, m.created_at, (SELECT COUNT(*) FROM alias a WHERE a.mailbox_id = m.id) AS nb_alias FROM mailbox m WHERE m.id = $1 AND m.user_id = $2",
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
        mailbox_id: &'a i64,
        user_id: &'a i64,
    ) -> MailboxRowQuery<'c, 'a, 's, C, MailboxRow, 2> {
        MailboxRowQuery {
            client,
            params: [mailbox_id, user_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<MailboxRowBorrowed, tokio_postgres::Error> {
                    Ok(MailboxRowBorrowed {
                        id: row.try_get(0)?,
                        email: row.try_get(1)?,
                        display_name: row.try_get(2)?,
                        verified: row.try_get(3)?,
                        enabled: row.try_get(4)?,
                        created_at: row.try_get(5)?,
                        nb_alias: row.try_get(6)?,
                    })
                },
            mapper: |it| MailboxRow::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        ByIdUserParams,
        MailboxRowQuery<'c, 'a, 's, C, MailboxRow, 2>,
        C,
    > for ByIdUserStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a ByIdUserParams,
    ) -> MailboxRowQuery<'c, 'a, 's, C, MailboxRow, 2> {
        self.bind(client, &params.mailbox_id, &params.user_id)
    }
}
pub struct ExistsVerifiedStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn exists_verified() -> ExistsVerifiedStmt {
    ExistsVerifiedStmt(
        "SELECT 1 AS one FROM mailbox WHERE id = $1 AND user_id = $2 AND enabled AND verified",
        None,
    )
}
impl ExistsVerifiedStmt {
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
        mailbox_id: &'a i64,
        user_id: &'a i64,
    ) -> I32Query<'c, 'a, 's, C, i32, 2> {
        I32Query {
            client,
            params: [mailbox_id, user_id],
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
        ExistsVerifiedParams,
        I32Query<'c, 'a, 's, C, i32, 2>,
        C,
    > for ExistsVerifiedStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a ExistsVerifiedParams,
    ) -> I32Query<'c, 'a, 's, C, i32, 2> {
        self.bind(client, &params.mailbox_id, &params.user_id)
    }
}
pub struct IdIfVerifiedStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn id_if_verified() -> IdIfVerifiedStmt {
    IdIfVerifiedStmt(
        "SELECT id FROM mailbox WHERE id = $1 AND user_id = $2 AND enabled AND verified",
        None,
    )
}
impl IdIfVerifiedStmt {
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
        mailbox_id: &'a i64,
        user_id: &'a i64,
    ) -> I64Query<'c, 'a, 's, C, i64, 2> {
        I64Query {
            client,
            params: [mailbox_id, user_id],
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
        IdIfVerifiedParams,
        I64Query<'c, 'a, 's, C, i64, 2>,
        C,
    > for IdIfVerifiedStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a IdIfVerifiedParams,
    ) -> I64Query<'c, 'a, 's, C, i64, 2> {
        self.bind(client, &params.mailbox_id, &params.user_id)
    }
}
pub struct FirstVerifiedForUserStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn first_verified_for_user() -> FirstVerifiedForUserStmt {
    FirstVerifiedForUserStmt(
        "SELECT id FROM mailbox WHERE user_id = $1 AND enabled AND verified ORDER BY id LIMIT 1",
        None,
    )
}
impl FirstVerifiedForUserStmt {
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
    ) -> I64Query<'c, 'a, 's, C, i64, 1> {
        I64Query {
            client,
            params: [user_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it,
        }
    }
}
pub struct IdForUserEmailStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn id_for_user_email() -> IdForUserEmailStmt {
    IdForUserEmailStmt(
        "SELECT id FROM mailbox WHERE user_id = $1 AND email = $2 AND enabled AND verified",
        None,
    )
}
impl IdForUserEmailStmt {
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
        email: &'a T1,
    ) -> I64Query<'c, 'a, 's, C, i64, 2> {
        I64Query {
            client,
            params: [user_id, email],
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
        IdForUserEmailParams<T1>,
        I64Query<'c, 'a, 's, C, i64, 2>,
        C,
    > for IdForUserEmailStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a IdForUserEmailParams<T1>,
    ) -> I64Query<'c, 'a, 's, C, i64, 2> {
        self.bind(client, &params.user_id, &params.email)
    }
}
pub struct EmailAndVerifiedStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn email_and_verified() -> EmailAndVerifiedStmt {
    EmailAndVerifiedStmt(
        "SELECT email::text AS email, verified FROM mailbox WHERE id = $1",
        None,
    )
}
impl EmailAndVerifiedStmt {
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
        mailbox_id: &'a i64,
    ) -> EmailAndVerifiedQuery<'c, 'a, 's, C, EmailAndVerified, 1> {
        EmailAndVerifiedQuery {
            client,
            params: [mailbox_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<EmailAndVerifiedBorrowed, tokio_postgres::Error> {
                Ok(EmailAndVerifiedBorrowed {
                    email: row.try_get(0)?,
                    verified: row.try_get(1)?,
                })
            },
            mapper: |it| EmailAndVerified::from(it),
        }
    }
}
pub struct VerifiedForUserStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn verified_for_user() -> VerifiedForUserStmt {
    VerifiedForUserStmt(
        "SELECT verified FROM mailbox WHERE id = $1 AND user_id = $2",
        None,
    )
}
impl VerifiedForUserStmt {
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
        mailbox_id: &'a i64,
        user_id: &'a i64,
    ) -> BoolQuery<'c, 'a, 's, C, bool, 2> {
        BoolQuery {
            client,
            params: [mailbox_id, user_id],
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
        VerifiedForUserParams,
        BoolQuery<'c, 'a, 's, C, bool, 2>,
        C,
    > for VerifiedForUserStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a VerifiedForUserParams,
    ) -> BoolQuery<'c, 'a, 's, C, bool, 2> {
        self.bind(client, &params.mailbox_id, &params.user_id)
    }
}
pub struct CreateStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn create() -> CreateStmt {
    CreateStmt(
        "INSERT INTO mailbox (user_id, email, display_name, verified) VALUES ($1, $2, $3, FALSE) RETURNING id",
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
        email: &'a T1,
        display_name: &'a Option<T2>,
    ) -> I64Query<'c, 'a, 's, C, i64, 3> {
        I64Query {
            client,
            params: [user_id, email, display_name],
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
        self.bind(client, &params.user_id, &params.email, &params.display_name)
    }
}
pub struct CreateVerifiedStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn create_verified() -> CreateVerifiedStmt {
    CreateVerifiedStmt(
        "INSERT INTO mailbox (user_id, email, display_name, verified) VALUES ($1, $2, $3, TRUE) RETURNING id",
        None,
    )
}
impl CreateVerifiedStmt {
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
        email: &'a T1,
        display_name: &'a Option<T2>,
    ) -> I64Query<'c, 'a, 's, C, i64, 3> {
        I64Query {
            client,
            params: [user_id, email, display_name],
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
        CreateVerifiedParams<T1, T2>,
        I64Query<'c, 'a, 's, C, i64, 3>,
        C,
    > for CreateVerifiedStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a CreateVerifiedParams<T1, T2>,
    ) -> I64Query<'c, 'a, 's, C, i64, 3> {
        self.bind(client, &params.user_id, &params.email, &params.display_name)
    }
}
pub struct SetDisplayNameStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn set_display_name() -> SetDisplayNameStmt {
    SetDisplayNameStmt(
        "UPDATE mailbox SET display_name = $1 WHERE id = $2 AND user_id = $3",
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
        mailbox_id: &'a i64,
        user_id: &'a i64,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(self.0, &[display_name, mailbox_id, user_id])
            .await
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
        Box::pin(self.bind(
            client,
            &params.display_name,
            &params.mailbox_id,
            &params.user_id,
        ))
    }
}
pub struct SetEnabledStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn set_enabled() -> SetEnabledStmt {
    SetEnabledStmt(
        "UPDATE mailbox SET enabled = $1 WHERE id = $2 AND user_id = $3",
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
        mailbox_id: &'a i64,
        user_id: &'a i64,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(self.0, &[enabled, mailbox_id, user_id])
            .await
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
        Box::pin(self.bind(client, &params.enabled, &params.mailbox_id, &params.user_id))
    }
}
pub struct SetVerifiedStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn set_verified() -> SetVerifiedStmt {
    SetVerifiedStmt("UPDATE mailbox SET verified = TRUE WHERE id = $1", None)
}
impl SetVerifiedStmt {
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
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[mailbox_id]).await
    }
}
pub struct DeleteStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn delete() -> DeleteStmt {
    DeleteStmt("DELETE FROM mailbox WHERE id = $1 AND user_id = $2", None)
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
        mailbox_id: &'a i64,
        user_id: &'a i64,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[mailbox_id, user_id]).await
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
        Box::pin(self.bind(client, &params.mailbox_id, &params.user_id))
    }
}
pub struct ListAdminStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn list_admin() -> ListAdminStmt {
    ListAdminStmt(
        "SELECT m.id, u.email::text AS user_email, m.email::text AS email, m.display_name, m.verified, m.enabled FROM mailbox m JOIN \"user\" u ON u.id = m.user_id ORDER BY m.user_id, m.id",
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
    ) -> MailboxAdminRowQuery<'c, 'a, 's, C, MailboxAdminRow, 0> {
        MailboxAdminRowQuery {
            client,
            params: [],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<MailboxAdminRowBorrowed, tokio_postgres::Error> {
                Ok(MailboxAdminRowBorrowed {
                    id: row.try_get(0)?,
                    user_email: row.try_get(1)?,
                    email: row.try_get(2)?,
                    display_name: row.try_get(3)?,
                    verified: row.try_get(4)?,
                    enabled: row.try_get(5)?,
                })
            },
            mapper: |it| MailboxAdminRow::from(it),
        }
    }
}
pub struct ListAdminForUserStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn list_admin_for_user() -> ListAdminForUserStmt {
    ListAdminForUserStmt(
        "SELECT m.id, u.email::text AS user_email, m.email::text AS email, m.display_name, m.verified, m.enabled FROM mailbox m JOIN \"user\" u ON u.id = m.user_id WHERE m.user_id = $1 ORDER BY m.id",
        None,
    )
}
impl ListAdminForUserStmt {
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
    ) -> MailboxAdminRowQuery<'c, 'a, 's, C, MailboxAdminRow, 1> {
        MailboxAdminRowQuery {
            client,
            params: [user_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<MailboxAdminRowBorrowed, tokio_postgres::Error> {
                Ok(MailboxAdminRowBorrowed {
                    id: row.try_get(0)?,
                    user_email: row.try_get(1)?,
                    email: row.try_get(2)?,
                    display_name: row.try_get(3)?,
                    verified: row.try_get(4)?,
                    enabled: row.try_get(5)?,
                })
            },
            mapper: |it| MailboxAdminRow::from(it),
        }
    }
}
