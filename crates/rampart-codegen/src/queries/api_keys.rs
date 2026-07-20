// This file was generated with `cornucopia`. Do not modify.

#[derive(Debug)]
pub struct CreateExtensionParams<
    T1: crate::StringSql,
    T2: crate::BytesSql,
    T3: crate::StringSql,
    T4: crate::ArraySql<Item = T3>,
    T5: crate::StringSql,
> {
    pub user_id: i64,
    pub name: T1,
    pub key_hash: T2,
    pub scopes: T4,
    pub token_prefix: Option<T5>,
    pub expires_at: Option<time::OffsetDateTime>,
}
#[derive(Clone, Copy, Debug)]
pub struct RevokeForUserParams {
    pub api_key_id: i64,
    pub user_id: i64,
}
#[derive(Clone, Copy, Debug)]
pub struct RevokeSelfParams {
    pub api_key_id: i64,
    pub user_id: i64,
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct LookupWithUser {
    pub api_key_id: i64,
    pub user_id: i64,
    pub is_admin: bool,
    pub scopes: Vec<String>,
}
pub struct LookupWithUserBorrowed<'a> {
    pub api_key_id: i64,
    pub user_id: i64,
    pub is_admin: bool,
    pub scopes: crate::ArrayIterator<'a, &'a str>,
}
impl<'a> From<LookupWithUserBorrowed<'a>> for LookupWithUser {
    fn from(
        LookupWithUserBorrowed {
            api_key_id,
            user_id,
            is_admin,
            scopes,
        }: LookupWithUserBorrowed<'a>,
    ) -> Self {
        Self {
            api_key_id,
            user_id,
            is_admin,
            scopes: scopes.map(|v| v.into()).collect(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ApiKeyRow {
    pub id: i64,
    pub name: String,
    pub scopes: Vec<String>,
    pub kind: String,
    pub token_prefix: Option<String>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub last_used_at: Option<time::OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub revoked_at: Option<time::OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: time::OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub expires_at: Option<time::OffsetDateTime>,
}
pub struct ApiKeyRowBorrowed<'a> {
    pub id: i64,
    pub name: &'a str,
    pub scopes: crate::ArrayIterator<'a, &'a str>,
    pub kind: &'a str,
    pub token_prefix: Option<&'a str>,
    pub last_used_at: Option<time::OffsetDateTime>,
    pub revoked_at: Option<time::OffsetDateTime>,
    pub created_at: time::OffsetDateTime,
    pub expires_at: Option<time::OffsetDateTime>,
}
impl<'a> From<ApiKeyRowBorrowed<'a>> for ApiKeyRow {
    fn from(
        ApiKeyRowBorrowed {
            id,
            name,
            scopes,
            kind,
            token_prefix,
            last_used_at,
            revoked_at,
            created_at,
            expires_at,
        }: ApiKeyRowBorrowed<'a>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            scopes: scopes.map(|v| v.into()).collect(),
            kind: kind.into(),
            token_prefix: token_prefix.map(|v| v.into()),
            last_used_at,
            revoked_at,
            created_at,
            expires_at,
        }
    }
}
use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
pub struct LookupWithUserQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<LookupWithUserBorrowed, tokio_postgres::Error>,
    mapper: fn(LookupWithUserBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> LookupWithUserQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(LookupWithUserBorrowed) -> R,
    ) -> LookupWithUserQuery<'c, 'a, 's, C, R, N> {
        LookupWithUserQuery {
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
pub struct ApiKeyRowQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<ApiKeyRowBorrowed, tokio_postgres::Error>,
    mapper: fn(ApiKeyRowBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> ApiKeyRowQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(self, mapper: fn(ApiKeyRowBorrowed) -> R) -> ApiKeyRowQuery<'c, 'a, 's, C, R, N> {
        ApiKeyRowQuery {
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
pub struct LookupWithUserStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn lookup_with_user() -> LookupWithUserStmt {
    LookupWithUserStmt(
        "SELECT k.id AS api_key_id, k.user_id, u.is_admin, k.scopes FROM api_key k JOIN \"user\" u ON u.id = k.user_id WHERE k.key_hash = $1 AND k.revoked_at IS NULL AND (k.expires_at IS NULL OR k.expires_at > now()) AND u.enabled",
        None,
    )
}
impl LookupWithUserStmt {
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
        key_hash: &'a T1,
    ) -> LookupWithUserQuery<'c, 'a, 's, C, LookupWithUser, 1> {
        LookupWithUserQuery {
            client,
            params: [key_hash],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<LookupWithUserBorrowed, tokio_postgres::Error> {
                Ok(LookupWithUserBorrowed {
                    api_key_id: row.try_get(0)?,
                    user_id: row.try_get(1)?,
                    is_admin: row.try_get(2)?,
                    scopes: row.try_get(3)?,
                })
            },
            mapper: |it| LookupWithUser::from(it),
        }
    }
}
pub struct BumpLastUsedStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn bump_last_used() -> BumpLastUsedStmt {
    BumpLastUsedStmt(
        "UPDATE api_key SET last_used_at = now() WHERE key_hash = $1",
        None,
    )
}
impl BumpLastUsedStmt {
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
        key_hash: &'a T1,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[key_hash]).await
    }
}
pub struct RevokeAllForUserStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn revoke_all_for_user() -> RevokeAllForUserStmt {
    RevokeAllForUserStmt(
        "UPDATE api_key SET revoked_at = now() WHERE user_id = $1 AND revoked_at IS NULL",
        None,
    )
}
impl RevokeAllForUserStmt {
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
pub struct ListForUserStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn list_for_user() -> ListForUserStmt {
    ListForUserStmt(
        "SELECT id, name, scopes, kind, token_prefix, last_used_at, revoked_at, created_at, expires_at FROM api_key WHERE user_id = $1 ORDER BY revoked_at NULLS FIRST, id DESC",
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
    ) -> ApiKeyRowQuery<'c, 'a, 's, C, ApiKeyRow, 1> {
        ApiKeyRowQuery {
            client,
            params: [user_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<ApiKeyRowBorrowed, tokio_postgres::Error> {
                    Ok(ApiKeyRowBorrowed {
                        id: row.try_get(0)?,
                        name: row.try_get(1)?,
                        scopes: row.try_get(2)?,
                        kind: row.try_get(3)?,
                        token_prefix: row.try_get(4)?,
                        last_used_at: row.try_get(5)?,
                        revoked_at: row.try_get(6)?,
                        created_at: row.try_get(7)?,
                        expires_at: row.try_get(8)?,
                    })
                },
            mapper: |it| ApiKeyRow::from(it),
        }
    }
}
pub struct CreateExtensionStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn create_extension() -> CreateExtensionStmt {
    CreateExtensionStmt(
        "INSERT INTO api_key (user_id, name, key_hash, scopes, kind, token_prefix, expires_at) VALUES ($1, $2, $3, $4, 'extension', $5, $6) RETURNING id, name, scopes, kind, token_prefix, last_used_at, revoked_at, created_at, expires_at",
        None,
    )
}
impl CreateExtensionStmt {
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
        T2: crate::BytesSql,
        T3: crate::StringSql,
        T4: crate::ArraySql<Item = T3>,
        T5: crate::StringSql,
    >(
        &'s self,
        client: &'c C,
        user_id: &'a i64,
        name: &'a T1,
        key_hash: &'a T2,
        scopes: &'a T4,
        token_prefix: &'a Option<T5>,
        expires_at: &'a Option<time::OffsetDateTime>,
    ) -> ApiKeyRowQuery<'c, 'a, 's, C, ApiKeyRow, 6> {
        ApiKeyRowQuery {
            client,
            params: [user_id, name, key_hash, scopes, token_prefix, expires_at],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<ApiKeyRowBorrowed, tokio_postgres::Error> {
                    Ok(ApiKeyRowBorrowed {
                        id: row.try_get(0)?,
                        name: row.try_get(1)?,
                        scopes: row.try_get(2)?,
                        kind: row.try_get(3)?,
                        token_prefix: row.try_get(4)?,
                        last_used_at: row.try_get(5)?,
                        revoked_at: row.try_get(6)?,
                        created_at: row.try_get(7)?,
                        expires_at: row.try_get(8)?,
                    })
                },
            mapper: |it| ApiKeyRow::from(it),
        }
    }
}
impl<
    'c,
    'a,
    's,
    C: GenericClient,
    T1: crate::StringSql,
    T2: crate::BytesSql,
    T3: crate::StringSql,
    T4: crate::ArraySql<Item = T3>,
    T5: crate::StringSql,
>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        CreateExtensionParams<T1, T2, T3, T4, T5>,
        ApiKeyRowQuery<'c, 'a, 's, C, ApiKeyRow, 6>,
        C,
    > for CreateExtensionStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a CreateExtensionParams<T1, T2, T3, T4, T5>,
    ) -> ApiKeyRowQuery<'c, 'a, 's, C, ApiKeyRow, 6> {
        self.bind(
            client,
            &params.user_id,
            &params.name,
            &params.key_hash,
            &params.scopes,
            &params.token_prefix,
            &params.expires_at,
        )
    }
}
pub struct RevokeForUserStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn revoke_for_user() -> RevokeForUserStmt {
    RevokeForUserStmt(
        "UPDATE api_key SET revoked_at = now() WHERE id = $1 AND user_id = $2 AND revoked_at IS NULL",
        None,
    )
}
impl RevokeForUserStmt {
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
        api_key_id: &'a i64,
        user_id: &'a i64,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[api_key_id, user_id]).await
    }
}
impl<'a, C: GenericClient + Send + Sync>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        RevokeForUserParams,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for RevokeForUserStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a RevokeForUserParams,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.api_key_id, &params.user_id))
    }
}
pub struct RevokeSelfStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn revoke_self() -> RevokeSelfStmt {
    RevokeSelfStmt(
        "UPDATE api_key SET revoked_at = now() WHERE id = $1 AND user_id = $2 AND revoked_at IS NULL",
        None,
    )
}
impl RevokeSelfStmt {
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
        api_key_id: &'a i64,
        user_id: &'a i64,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[api_key_id, user_id]).await
    }
}
impl<'a, C: GenericClient + Send + Sync>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        RevokeSelfParams,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for RevokeSelfStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a RevokeSelfParams,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.api_key_id, &params.user_id))
    }
}
