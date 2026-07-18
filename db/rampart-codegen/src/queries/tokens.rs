// This file was generated with `cornucopia`. Do not modify.

#[derive(Debug)]
pub struct InviteCreateParams<T1: crate::BytesSql, T2: crate::StringSql> {
    pub token_hash: T1,
    pub preset_email: Option<T2>,
    pub expires_at: time::OffsetDateTime,
}
#[derive(Debug)]
pub struct InviteClaimParams<T1: crate::BytesSql, T2: crate::StringSql> {
    pub token_hash: T1,
    pub email: T2,
}
#[derive(Debug)]
pub struct InviteFailureParams<T1: crate::BytesSql, T2: crate::StringSql> {
    pub token_hash: T1,
    pub email: T2,
}
#[derive(Debug)]
pub struct InviteSetUsedByParams<T1: crate::BytesSql> {
    pub user_id: i64,
    pub token_hash: T1,
}
#[derive(Debug)]
pub struct PasswordResetCreateParams<T1: crate::BytesSql> {
    pub token_hash: T1,
    pub user_id: i64,
    pub expires_at: time::OffsetDateTime,
}
#[derive(Debug)]
pub struct EmailChangeCreateParams<T1: crate::BytesSql, T2: crate::StringSql> {
    pub token_hash: T1,
    pub user_id: i64,
    pub new_email: T2,
    pub expires_at: time::OffsetDateTime,
}
#[derive(Debug)]
pub struct MailboxVerifyCreateParams<T1: crate::BytesSql> {
    pub token_hash: T1,
    pub mailbox_id: i64,
    pub expires_at: time::OffsetDateTime,
}
#[derive(Debug, Clone, PartialEq, Copy, serde::Serialize)]
pub struct InviteFailure {
    pub used: bool,
    pub expired: bool,
    pub email_mismatch: bool,
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct EmailChangeClaim {
    pub user_id: i64,
    pub new_email: String,
}
pub struct EmailChangeClaimBorrowed<'a> {
    pub user_id: i64,
    pub new_email: &'a str,
}
impl<'a> From<EmailChangeClaimBorrowed<'a>> for EmailChangeClaim {
    fn from(EmailChangeClaimBorrowed { user_id, new_email }: EmailChangeClaimBorrowed<'a>) -> Self {
        Self {
            user_id,
            new_email: new_email.into(),
        }
    }
}
use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
pub struct Vecu8Query<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<&[u8], tokio_postgres::Error>,
    mapper: fn(&[u8]) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> Vecu8Query<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(self, mapper: fn(&[u8]) -> R) -> Vecu8Query<'c, 'a, 's, C, R, N> {
        Vecu8Query {
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
pub struct InviteFailureQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<InviteFailure, tokio_postgres::Error>,
    mapper: fn(InviteFailure) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> InviteFailureQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(self, mapper: fn(InviteFailure) -> R) -> InviteFailureQuery<'c, 'a, 's, C, R, N> {
        InviteFailureQuery {
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
pub struct EmailChangeClaimQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<EmailChangeClaimBorrowed, tokio_postgres::Error>,
    mapper: fn(EmailChangeClaimBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> EmailChangeClaimQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(EmailChangeClaimBorrowed) -> R,
    ) -> EmailChangeClaimQuery<'c, 'a, 's, C, R, N> {
        EmailChangeClaimQuery {
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
pub struct InviteCreateStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn invite_create() -> InviteCreateStmt {
    InviteCreateStmt(
        "INSERT INTO invite_token (token_hash, preset_email, expires_at) VALUES ($1, $2, $3)",
        None,
    )
}
impl InviteCreateStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient, T1: crate::BytesSql, T2: crate::StringSql>(
        &'s self,
        client: &'c C,
        token_hash: &'a T1,
        preset_email: &'a Option<T2>,
        expires_at: &'a time::OffsetDateTime,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(self.0, &[token_hash, preset_email, expires_at])
            .await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::BytesSql, T2: crate::StringSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        InviteCreateParams<T1, T2>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for InviteCreateStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a InviteCreateParams<T1, T2>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.token_hash,
            &params.preset_email,
            &params.expires_at,
        ))
    }
}
pub struct InviteClaimStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn invite_claim() -> InviteClaimStmt {
    InviteClaimStmt(
        "UPDATE invite_token SET used_at = now() WHERE token_hash = $1 AND used_at IS NULL AND expires_at > now() AND (preset_email IS NULL OR preset_email = $2::CITEXT) RETURNING token_hash",
        None,
    )
}
impl InviteClaimStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient, T1: crate::BytesSql, T2: crate::StringSql>(
        &'s self,
        client: &'c C,
        token_hash: &'a T1,
        email: &'a T2,
    ) -> Vecu8Query<'c, 'a, 's, C, Vec<u8>, 2> {
        Vecu8Query {
            client,
            params: [token_hash, email],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it.into(),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::BytesSql, T2: crate::StringSql>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        InviteClaimParams<T1, T2>,
        Vecu8Query<'c, 'a, 's, C, Vec<u8>, 2>,
        C,
    > for InviteClaimStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a InviteClaimParams<T1, T2>,
    ) -> Vecu8Query<'c, 'a, 's, C, Vec<u8>, 2> {
        self.bind(client, &params.token_hash, &params.email)
    }
}
pub struct InviteFailureStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn invite_failure() -> InviteFailureStmt {
    InviteFailureStmt(
        "WITH invite AS ( SELECT used_at, expires_at, preset_email FROM invite_token WHERE token_hash = $1 ) SELECT used_at IS NOT NULL AS used, expires_at <= now() AS expired, preset_email IS NOT NULL AND preset_email <> $2::CITEXT AS email_mismatch FROM invite",
        None,
    )
}
impl InviteFailureStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient, T1: crate::BytesSql, T2: crate::StringSql>(
        &'s self,
        client: &'c C,
        token_hash: &'a T1,
        email: &'a T2,
    ) -> InviteFailureQuery<'c, 'a, 's, C, InviteFailure, 2> {
        InviteFailureQuery {
            client,
            params: [token_hash, email],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row: &tokio_postgres::Row| -> Result<InviteFailure, tokio_postgres::Error> {
                Ok(InviteFailure {
                    used: row.try_get(0)?,
                    expired: row.try_get(1)?,
                    email_mismatch: row.try_get(2)?,
                })
            },
            mapper: |it| InviteFailure::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::BytesSql, T2: crate::StringSql>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        InviteFailureParams<T1, T2>,
        InviteFailureQuery<'c, 'a, 's, C, InviteFailure, 2>,
        C,
    > for InviteFailureStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a InviteFailureParams<T1, T2>,
    ) -> InviteFailureQuery<'c, 'a, 's, C, InviteFailure, 2> {
        self.bind(client, &params.token_hash, &params.email)
    }
}
pub struct InviteSetUsedByStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn invite_set_used_by() -> InviteSetUsedByStmt {
    InviteSetUsedByStmt(
        "UPDATE invite_token SET used_by = $1 WHERE token_hash = $2",
        None,
    )
}
impl InviteSetUsedByStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient, T1: crate::BytesSql>(
        &'s self,
        client: &'c C,
        user_id: &'a i64,
        token_hash: &'a T1,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[user_id, token_hash]).await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::BytesSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        InviteSetUsedByParams<T1>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for InviteSetUsedByStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a InviteSetUsedByParams<T1>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.user_id, &params.token_hash))
    }
}
pub struct PasswordResetCreateStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn password_reset_create() -> PasswordResetCreateStmt {
    PasswordResetCreateStmt(
        "INSERT INTO password_reset_token (token_hash, user_id, expires_at) VALUES ($1, $2, $3)",
        None,
    )
}
impl PasswordResetCreateStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient, T1: crate::BytesSql>(
        &'s self,
        client: &'c C,
        token_hash: &'a T1,
        user_id: &'a i64,
        expires_at: &'a time::OffsetDateTime,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(self.0, &[token_hash, user_id, expires_at])
            .await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::BytesSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        PasswordResetCreateParams<T1>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for PasswordResetCreateStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a PasswordResetCreateParams<T1>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.token_hash,
            &params.user_id,
            &params.expires_at,
        ))
    }
}
pub struct PasswordResetClaimStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn password_reset_claim() -> PasswordResetClaimStmt {
    PasswordResetClaimStmt(
        "UPDATE password_reset_token SET used_at = now() WHERE token_hash = $1 AND used_at IS NULL AND expires_at > now() RETURNING user_id",
        None,
    )
}
impl PasswordResetClaimStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient, T1: crate::BytesSql>(
        &'s self,
        client: &'c C,
        token_hash: &'a T1,
    ) -> I64Query<'c, 'a, 's, C, i64, 1> {
        I64Query {
            client,
            params: [token_hash],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it,
        }
    }
}
pub struct EmailChangeCreateStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn email_change_create() -> EmailChangeCreateStmt {
    EmailChangeCreateStmt(
        "INSERT INTO email_change_token (token_hash, user_id, new_email, expires_at) VALUES ($1, $2, $3, $4)",
        None,
    )
}
impl EmailChangeCreateStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient, T1: crate::BytesSql, T2: crate::StringSql>(
        &'s self,
        client: &'c C,
        token_hash: &'a T1,
        user_id: &'a i64,
        new_email: &'a T2,
        expires_at: &'a time::OffsetDateTime,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(self.0, &[token_hash, user_id, new_email, expires_at])
            .await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::BytesSql, T2: crate::StringSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        EmailChangeCreateParams<T1, T2>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for EmailChangeCreateStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a EmailChangeCreateParams<T1, T2>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.token_hash,
            &params.user_id,
            &params.new_email,
            &params.expires_at,
        ))
    }
}
pub struct EmailChangeClaimStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn email_change_claim() -> EmailChangeClaimStmt {
    EmailChangeClaimStmt(
        "UPDATE email_change_token SET used_at = now() WHERE token_hash = $1 AND used_at IS NULL AND expires_at > now() RETURNING user_id, new_email::text AS new_email",
        None,
    )
}
impl EmailChangeClaimStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient, T1: crate::BytesSql>(
        &'s self,
        client: &'c C,
        token_hash: &'a T1,
    ) -> EmailChangeClaimQuery<'c, 'a, 's, C, EmailChangeClaim, 1> {
        EmailChangeClaimQuery {
            client,
            params: [token_hash],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<EmailChangeClaimBorrowed, tokio_postgres::Error> {
                Ok(EmailChangeClaimBorrowed {
                    user_id: row.try_get(0)?,
                    new_email: row.try_get(1)?,
                })
            },
            mapper: |it| EmailChangeClaim::from(it),
        }
    }
}
pub struct MailboxVerifyCreateStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn mailbox_verify_create() -> MailboxVerifyCreateStmt {
    MailboxVerifyCreateStmt(
        "INSERT INTO mailbox_verify_token (token_hash, mailbox_id, expires_at) VALUES ($1, $2, $3)",
        None,
    )
}
impl MailboxVerifyCreateStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient, T1: crate::BytesSql>(
        &'s self,
        client: &'c C,
        token_hash: &'a T1,
        mailbox_id: &'a i64,
        expires_at: &'a time::OffsetDateTime,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(self.0, &[token_hash, mailbox_id, expires_at])
            .await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::BytesSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        MailboxVerifyCreateParams<T1>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for MailboxVerifyCreateStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a MailboxVerifyCreateParams<T1>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.token_hash,
            &params.mailbox_id,
            &params.expires_at,
        ))
    }
}
pub struct MailboxVerifyClaimStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn mailbox_verify_claim() -> MailboxVerifyClaimStmt {
    MailboxVerifyClaimStmt(
        "UPDATE mailbox_verify_token SET used_at = now() WHERE token_hash = $1 AND used_at IS NULL AND expires_at > now() RETURNING mailbox_id",
        None,
    )
}
impl MailboxVerifyClaimStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient, T1: crate::BytesSql>(
        &'s self,
        client: &'c C,
        token_hash: &'a T1,
    ) -> I64Query<'c, 'a, 's, C, i64, 1> {
        I64Query {
            client,
            params: [token_hash],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it,
        }
    }
}
