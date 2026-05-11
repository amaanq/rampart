// This file was generated with `clorinde`. Do not modify.

#[derive(Debug)]
pub struct BumpLastSeenParams<T1: crate::BytesSql> {
    pub expires_at: time::OffsetDateTime,
    pub session_id: T1,
}
#[derive(Debug)]
pub struct CreateParams<T1: crate::BytesSql, T2: crate::StringSql> {
    pub session_id: T1,
    pub user_id: i64,
    pub expires_at: time::OffsetDateTime,
    pub user_agent: Option<T2>,
}
#[derive(Debug, Clone, PartialEq, Copy, serde::Serialize)]
pub struct LookupWithUser {
    pub user_id: i64,
    #[serde(with = "time::serde::rfc3339")]
    pub expires_at: time::OffsetDateTime,
    pub is_admin: bool,
    pub enabled: bool,
}
use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
pub struct LookupWithUserQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<LookupWithUser, tokio_postgres::Error>,
    mapper: fn(LookupWithUser) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> LookupWithUserQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(LookupWithUser) -> R,
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
pub struct LookupWithUserStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn lookup_with_user() -> LookupWithUserStmt {
    LookupWithUserStmt(
        "SELECT s.user_id, s.expires_at, u.is_admin, u.enabled FROM session s JOIN \"user\" u ON u.id = s.user_id WHERE s.id = $1",
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
        session_id: &'a T1,
    ) -> LookupWithUserQuery<'c, 'a, 's, C, LookupWithUser, 1> {
        LookupWithUserQuery {
            client,
            params: [session_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<LookupWithUser, tokio_postgres::Error> {
                    Ok(LookupWithUser {
                        user_id: row.try_get(0)?,
                        expires_at: row.try_get(1)?,
                        is_admin: row.try_get(2)?,
                        enabled: row.try_get(3)?,
                    })
                },
            mapper: |it| LookupWithUser::from(it),
        }
    }
}
pub struct DeleteByIdStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn delete_by_id() -> DeleteByIdStmt {
    DeleteByIdStmt("DELETE FROM session WHERE id = $1", None)
}
impl DeleteByIdStmt {
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
        session_id: &'a T1,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[session_id]).await
    }
}
pub struct DeleteByUserStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn delete_by_user() -> DeleteByUserStmt {
    DeleteByUserStmt("DELETE FROM session WHERE user_id = $1", None)
}
impl DeleteByUserStmt {
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
pub struct BumpLastSeenStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn bump_last_seen() -> BumpLastSeenStmt {
    BumpLastSeenStmt(
        "UPDATE session SET last_seen_at = now(), expires_at = $1 WHERE id = $2",
        None,
    )
}
impl BumpLastSeenStmt {
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
        expires_at: &'a time::OffsetDateTime,
        session_id: &'a T1,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[expires_at, session_id]).await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::BytesSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        BumpLastSeenParams<T1>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for BumpLastSeenStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a BumpLastSeenParams<T1>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.expires_at, &params.session_id))
    }
}
pub struct CreateStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn create() -> CreateStmt {
    CreateStmt(
        "INSERT INTO session (id, user_id, expires_at, user_agent) VALUES ($1, $2, $3, $4)",
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
    pub async fn bind<'c, 'a, 's, C: GenericClient, T1: crate::BytesSql, T2: crate::StringSql>(
        &'s self,
        client: &'c C,
        session_id: &'a T1,
        user_id: &'a i64,
        expires_at: &'a time::OffsetDateTime,
        user_agent: &'a Option<T2>,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(self.0, &[session_id, user_id, expires_at, user_agent])
            .await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::BytesSql, T2: crate::StringSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        CreateParams<T1, T2>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for CreateStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a CreateParams<T1, T2>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.session_id,
            &params.user_id,
            &params.expires_at,
            &params.user_agent,
        ))
    }
}
