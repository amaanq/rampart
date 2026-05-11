// This file was generated with `clorinde`. Do not modify.

#[derive(Debug)]
pub struct CeremonyInsertRegisterParams<T1: crate::BytesSql, T2: crate::BytesSql> {
    pub id: T1,
    pub user_id: Option<i64>,
    pub state_blob: T2,
    pub expires_at: time::OffsetDateTime,
}
#[derive(Debug)]
pub struct CeremonyInsertAuthParams<T1: crate::BytesSql, T2: crate::BytesSql> {
    pub id: T1,
    pub user_id: Option<i64>,
    pub state_blob: T2,
    pub expires_at: time::OffsetDateTime,
}
#[derive(Debug)]
pub struct CeremonyConsumeRegisterParams<T1: crate::BytesSql> {
    pub id: T1,
    pub user_id: i64,
}
#[derive(Debug)]
pub struct CredentialInsertParams<T1: crate::BytesSql, T2: crate::BytesSql, T3: crate::StringSql> {
    pub user_id: i64,
    pub credential_id: T1,
    pub credential_blob: T2,
    pub name: T3,
}
#[derive(Debug)]
pub struct CredentialUpdateBlobAndCountParams<T1: crate::BytesSql, T2: crate::BytesSql> {
    pub sign_count: i32,
    pub credential_blob: T1,
    pub credential_id: T2,
}
#[derive(Debug)]
pub struct CredentialUpdateCountOnlyParams<T1: crate::BytesSql> {
    pub sign_count: i32,
    pub credential_id: T1,
}
#[derive(Clone, Copy, Debug)]
pub struct DeleteForUserParams {
    pub credential_pk: i64,
    pub user_id: i64,
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct CredentialForUpdate {
    pub sign_count: i32,
    pub credential_blob: Vec<u8>,
}
pub struct CredentialForUpdateBorrowed<'a> {
    pub sign_count: i32,
    pub credential_blob: &'a [u8],
}
impl<'a> From<CredentialForUpdateBorrowed<'a>> for CredentialForUpdate {
    fn from(
        CredentialForUpdateBorrowed {
            sign_count,
            credential_blob,
        }: CredentialForUpdateBorrowed<'a>,
    ) -> Self {
        Self {
            sign_count,
            credential_blob: credential_blob.into(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ListForUser {
    pub id: i64,
    pub name: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: time::OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub last_used_at: Option<time::OffsetDateTime>,
}
pub struct ListForUserBorrowed<'a> {
    pub id: i64,
    pub name: &'a str,
    pub created_at: time::OffsetDateTime,
    pub last_used_at: Option<time::OffsetDateTime>,
}
impl<'a> From<ListForUserBorrowed<'a>> for ListForUser {
    fn from(
        ListForUserBorrowed {
            id,
            name,
            created_at,
            last_used_at,
        }: ListForUserBorrowed<'a>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            created_at,
            last_used_at,
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
pub struct CredentialForUpdateQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor:
        fn(&tokio_postgres::Row) -> Result<CredentialForUpdateBorrowed, tokio_postgres::Error>,
    mapper: fn(CredentialForUpdateBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> CredentialForUpdateQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(CredentialForUpdateBorrowed) -> R,
    ) -> CredentialForUpdateQuery<'c, 'a, 's, C, R, N> {
        CredentialForUpdateQuery {
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
pub struct ListForUserQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<ListForUserBorrowed, tokio_postgres::Error>,
    mapper: fn(ListForUserBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> ListForUserQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(ListForUserBorrowed) -> R,
    ) -> ListForUserQuery<'c, 'a, 's, C, R, N> {
        ListForUserQuery {
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
pub struct CeremonyInsertRegisterStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn ceremony_insert_register() -> CeremonyInsertRegisterStmt {
    CeremonyInsertRegisterStmt(
        "INSERT INTO webauthn_ceremony (id, user_id, kind, state_blob, expires_at) VALUES ($1, $2, 'register', $3, $4)",
        None,
    )
}
impl CeremonyInsertRegisterStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient, T1: crate::BytesSql, T2: crate::BytesSql>(
        &'s self,
        client: &'c C,
        id: &'a T1,
        user_id: &'a Option<i64>,
        state_blob: &'a T2,
        expires_at: &'a time::OffsetDateTime,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(self.0, &[id, user_id, state_blob, expires_at])
            .await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::BytesSql, T2: crate::BytesSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        CeremonyInsertRegisterParams<T1, T2>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for CeremonyInsertRegisterStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a CeremonyInsertRegisterParams<T1, T2>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.id,
            &params.user_id,
            &params.state_blob,
            &params.expires_at,
        ))
    }
}
pub struct CeremonyInsertAuthStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn ceremony_insert_auth() -> CeremonyInsertAuthStmt {
    CeremonyInsertAuthStmt(
        "INSERT INTO webauthn_ceremony (id, user_id, kind, state_blob, expires_at) VALUES ($1, $2, 'auth', $3, $4)",
        None,
    )
}
impl CeremonyInsertAuthStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient, T1: crate::BytesSql, T2: crate::BytesSql>(
        &'s self,
        client: &'c C,
        id: &'a T1,
        user_id: &'a Option<i64>,
        state_blob: &'a T2,
        expires_at: &'a time::OffsetDateTime,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(self.0, &[id, user_id, state_blob, expires_at])
            .await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::BytesSql, T2: crate::BytesSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        CeremonyInsertAuthParams<T1, T2>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for CeremonyInsertAuthStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a CeremonyInsertAuthParams<T1, T2>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.id,
            &params.user_id,
            &params.state_blob,
            &params.expires_at,
        ))
    }
}
pub struct CeremonyConsumeRegisterStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn ceremony_consume_register() -> CeremonyConsumeRegisterStmt {
    CeremonyConsumeRegisterStmt(
        "DELETE FROM webauthn_ceremony WHERE id = $1 AND user_id = $2 AND kind = 'register' AND expires_at > now() RETURNING state_blob",
        None,
    )
}
impl CeremonyConsumeRegisterStmt {
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
        id: &'a T1,
        user_id: &'a i64,
    ) -> Vecu8Query<'c, 'a, 's, C, Vec<u8>, 2> {
        Vecu8Query {
            client,
            params: [id, user_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it.into(),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::BytesSql>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        CeremonyConsumeRegisterParams<T1>,
        Vecu8Query<'c, 'a, 's, C, Vec<u8>, 2>,
        C,
    > for CeremonyConsumeRegisterStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a CeremonyConsumeRegisterParams<T1>,
    ) -> Vecu8Query<'c, 'a, 's, C, Vec<u8>, 2> {
        self.bind(client, &params.id, &params.user_id)
    }
}
pub struct CeremonyConsumeAuthStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn ceremony_consume_auth() -> CeremonyConsumeAuthStmt {
    CeremonyConsumeAuthStmt(
        "DELETE FROM webauthn_ceremony WHERE id = $1 AND kind = 'auth' AND expires_at > now() RETURNING state_blob",
        None,
    )
}
impl CeremonyConsumeAuthStmt {
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
        id: &'a T1,
    ) -> Vecu8Query<'c, 'a, 's, C, Vec<u8>, 1> {
        Vecu8Query {
            client,
            params: [id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it.into(),
        }
    }
}
pub struct CredentialsForUserStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn credentials_for_user() -> CredentialsForUserStmt {
    CredentialsForUserStmt(
        "SELECT credential_blob FROM webauthn_credential WHERE user_id = $1",
        None,
    )
}
impl CredentialsForUserStmt {
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
    ) -> Vecu8Query<'c, 'a, 's, C, Vec<u8>, 1> {
        Vecu8Query {
            client,
            params: [user_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it.into(),
        }
    }
}
pub struct CredentialInsertStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn credential_insert() -> CredentialInsertStmt {
    CredentialInsertStmt(
        "INSERT INTO webauthn_credential (user_id, credential_id, credential_blob, name) VALUES ($1, $2, $3, $4)",
        None,
    )
}
impl CredentialInsertStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<
        'c,
        'a,
        's,
        C: GenericClient,
        T1: crate::BytesSql,
        T2: crate::BytesSql,
        T3: crate::StringSql,
    >(
        &'s self,
        client: &'c C,
        user_id: &'a i64,
        credential_id: &'a T1,
        credential_blob: &'a T2,
        name: &'a T3,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(self.0, &[user_id, credential_id, credential_blob, name])
            .await
    }
}
impl<
    'a,
    C: GenericClient + Send + Sync,
    T1: crate::BytesSql,
    T2: crate::BytesSql,
    T3: crate::StringSql,
>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        CredentialInsertParams<T1, T2, T3>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for CredentialInsertStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a CredentialInsertParams<T1, T2, T3>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.user_id,
            &params.credential_id,
            &params.credential_blob,
            &params.name,
        ))
    }
}
pub struct CredentialForUpdateStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn credential_for_update() -> CredentialForUpdateStmt {
    CredentialForUpdateStmt(
        "SELECT sign_count, credential_blob FROM webauthn_credential WHERE credential_id = $1 FOR UPDATE",
        None,
    )
}
impl CredentialForUpdateStmt {
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
        credential_id: &'a T1,
    ) -> CredentialForUpdateQuery<'c, 'a, 's, C, CredentialForUpdate, 1> {
        CredentialForUpdateQuery {
            client,
            params: [credential_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<CredentialForUpdateBorrowed, tokio_postgres::Error> {
                Ok(CredentialForUpdateBorrowed {
                    sign_count: row.try_get(0)?,
                    credential_blob: row.try_get(1)?,
                })
            },
            mapper: |it| CredentialForUpdate::from(it),
        }
    }
}
pub struct CredentialUpdateBlobAndCountStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn credential_update_blob_and_count() -> CredentialUpdateBlobAndCountStmt {
    CredentialUpdateBlobAndCountStmt(
        "UPDATE webauthn_credential SET sign_count = GREATEST(sign_count, $1::int), credential_blob = $2, last_used_at = now() WHERE credential_id = $3",
        None,
    )
}
impl CredentialUpdateBlobAndCountStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient, T1: crate::BytesSql, T2: crate::BytesSql>(
        &'s self,
        client: &'c C,
        sign_count: &'a i32,
        credential_blob: &'a T1,
        credential_id: &'a T2,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(self.0, &[sign_count, credential_blob, credential_id])
            .await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::BytesSql, T2: crate::BytesSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        CredentialUpdateBlobAndCountParams<T1, T2>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for CredentialUpdateBlobAndCountStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a CredentialUpdateBlobAndCountParams<T1, T2>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.sign_count,
            &params.credential_blob,
            &params.credential_id,
        ))
    }
}
pub struct CredentialUpdateCountOnlyStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn credential_update_count_only() -> CredentialUpdateCountOnlyStmt {
    CredentialUpdateCountOnlyStmt(
        "UPDATE webauthn_credential SET sign_count = GREATEST(sign_count, $1::int), last_used_at = now() WHERE credential_id = $2",
        None,
    )
}
impl CredentialUpdateCountOnlyStmt {
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
        sign_count: &'a i32,
        credential_id: &'a T1,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[sign_count, credential_id]).await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::BytesSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        CredentialUpdateCountOnlyParams<T1>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for CredentialUpdateCountOnlyStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a CredentialUpdateCountOnlyParams<T1>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.sign_count, &params.credential_id))
    }
}
pub struct CredentialUserIdStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn credential_user_id() -> CredentialUserIdStmt {
    CredentialUserIdStmt(
        "SELECT c.user_id FROM webauthn_credential c JOIN \"user\" u ON u.id = c.user_id WHERE c.credential_id = $1 AND u.enabled",
        None,
    )
}
impl CredentialUserIdStmt {
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
        credential_id: &'a T1,
    ) -> I64Query<'c, 'a, 's, C, i64, 1> {
        I64Query {
            client,
            params: [credential_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it,
        }
    }
}
pub struct ListForUserStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn list_for_user() -> ListForUserStmt {
    ListForUserStmt(
        "SELECT id, name, created_at, last_used_at FROM webauthn_credential WHERE user_id = $1 ORDER BY id",
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
    ) -> ListForUserQuery<'c, 'a, 's, C, ListForUser, 1> {
        ListForUserQuery {
            client,
            params: [user_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<ListForUserBorrowed, tokio_postgres::Error> {
                    Ok(ListForUserBorrowed {
                        id: row.try_get(0)?,
                        name: row.try_get(1)?,
                        created_at: row.try_get(2)?,
                        last_used_at: row.try_get(3)?,
                    })
                },
            mapper: |it| ListForUser::from(it),
        }
    }
}
pub struct DeleteForUserStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn delete_for_user() -> DeleteForUserStmt {
    DeleteForUserStmt(
        "DELETE FROM webauthn_credential WHERE id = $1 AND user_id = $2",
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
        credential_pk: &'a i64,
        user_id: &'a i64,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[credential_pk, user_id]).await
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
        Box::pin(self.bind(client, &params.credential_pk, &params.user_id))
    }
}
