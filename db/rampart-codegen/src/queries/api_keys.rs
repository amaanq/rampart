// This file was generated with `clorinde`. Do not modify.

#[derive(Debug, Clone, PartialEq, Copy, serde::Serialize)]
pub struct LookupWithUser {
    pub user_id: i64,
    pub is_admin: bool,
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
        "SELECT k.user_id, u.is_admin FROM api_key k JOIN \"user\" u ON u.id = k.user_id WHERE k.key_hash = $1 AND k.revoked_at IS NULL AND u.enabled",
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
            extractor:
                |row: &tokio_postgres::Row| -> Result<LookupWithUser, tokio_postgres::Error> {
                    Ok(LookupWithUser {
                        user_id: row.try_get(0)?,
                        is_admin: row.try_get(1)?,
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
