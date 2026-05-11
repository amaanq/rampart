// This file was generated with `clorinde`. Do not modify.

use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
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
pub struct CountInviteTokenStaleStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn count_invite_token_stale() -> CountInviteTokenStaleStmt {
    CountInviteTokenStaleStmt(
        "SELECT count(*)::bigint AS n FROM invite_token WHERE used_at IS NOT NULL OR expires_at < now()",
        None,
    )
}
impl CountInviteTokenStaleStmt {
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
    ) -> I64Query<'c, 'a, 's, C, i64, 0> {
        I64Query {
            client,
            params: [],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it,
        }
    }
}
pub struct DeleteInviteTokenStaleStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn delete_invite_token_stale() -> DeleteInviteTokenStaleStmt {
    DeleteInviteTokenStaleStmt(
        "DELETE FROM invite_token WHERE used_at IS NOT NULL OR expires_at < now()",
        None,
    )
}
impl DeleteInviteTokenStaleStmt {
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
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[]).await
    }
}
pub struct CountPasswordResetTokenStaleStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn count_password_reset_token_stale() -> CountPasswordResetTokenStaleStmt {
    CountPasswordResetTokenStaleStmt(
        "SELECT count(*)::bigint AS n FROM password_reset_token WHERE used_at IS NOT NULL OR expires_at < now()",
        None,
    )
}
impl CountPasswordResetTokenStaleStmt {
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
    ) -> I64Query<'c, 'a, 's, C, i64, 0> {
        I64Query {
            client,
            params: [],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it,
        }
    }
}
pub struct DeletePasswordResetTokenStaleStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn delete_password_reset_token_stale() -> DeletePasswordResetTokenStaleStmt {
    DeletePasswordResetTokenStaleStmt(
        "DELETE FROM password_reset_token WHERE used_at IS NOT NULL OR expires_at < now()",
        None,
    )
}
impl DeletePasswordResetTokenStaleStmt {
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
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[]).await
    }
}
pub struct CountEmailChangeTokenStaleStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn count_email_change_token_stale() -> CountEmailChangeTokenStaleStmt {
    CountEmailChangeTokenStaleStmt(
        "SELECT count(*)::bigint AS n FROM email_change_token WHERE used_at IS NOT NULL OR expires_at < now()",
        None,
    )
}
impl CountEmailChangeTokenStaleStmt {
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
    ) -> I64Query<'c, 'a, 's, C, i64, 0> {
        I64Query {
            client,
            params: [],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it,
        }
    }
}
pub struct DeleteEmailChangeTokenStaleStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn delete_email_change_token_stale() -> DeleteEmailChangeTokenStaleStmt {
    DeleteEmailChangeTokenStaleStmt(
        "DELETE FROM email_change_token WHERE used_at IS NOT NULL OR expires_at < now()",
        None,
    )
}
impl DeleteEmailChangeTokenStaleStmt {
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
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[]).await
    }
}
pub struct CountMailboxVerifyTokenStaleStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn count_mailbox_verify_token_stale() -> CountMailboxVerifyTokenStaleStmt {
    CountMailboxVerifyTokenStaleStmt(
        "SELECT count(*)::bigint AS n FROM mailbox_verify_token WHERE used_at IS NOT NULL OR expires_at < now()",
        None,
    )
}
impl CountMailboxVerifyTokenStaleStmt {
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
    ) -> I64Query<'c, 'a, 's, C, i64, 0> {
        I64Query {
            client,
            params: [],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it,
        }
    }
}
pub struct DeleteMailboxVerifyTokenStaleStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn delete_mailbox_verify_token_stale() -> DeleteMailboxVerifyTokenStaleStmt {
    DeleteMailboxVerifyTokenStaleStmt(
        "DELETE FROM mailbox_verify_token WHERE used_at IS NOT NULL OR expires_at < now()",
        None,
    )
}
impl DeleteMailboxVerifyTokenStaleStmt {
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
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[]).await
    }
}
pub struct CountWebauthnCeremonyStaleStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn count_webauthn_ceremony_stale() -> CountWebauthnCeremonyStaleStmt {
    CountWebauthnCeremonyStaleStmt(
        "SELECT count(*)::bigint AS n FROM webauthn_ceremony WHERE expires_at < now()",
        None,
    )
}
impl CountWebauthnCeremonyStaleStmt {
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
    ) -> I64Query<'c, 'a, 's, C, i64, 0> {
        I64Query {
            client,
            params: [],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it,
        }
    }
}
pub struct DeleteWebauthnCeremonyStaleStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn delete_webauthn_ceremony_stale() -> DeleteWebauthnCeremonyStaleStmt {
    DeleteWebauthnCeremonyStaleStmt(
        "DELETE FROM webauthn_ceremony WHERE expires_at < now()",
        None,
    )
}
impl DeleteWebauthnCeremonyStaleStmt {
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
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[]).await
    }
}
pub struct CountSessionStaleStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn count_session_stale() -> CountSessionStaleStmt {
    CountSessionStaleStmt(
        "SELECT count(*)::bigint AS n FROM session WHERE expires_at < now()",
        None,
    )
}
impl CountSessionStaleStmt {
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
    ) -> I64Query<'c, 'a, 's, C, i64, 0> {
        I64Query {
            client,
            params: [],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it,
        }
    }
}
pub struct DeleteSessionStaleStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn delete_session_stale() -> DeleteSessionStaleStmt {
    DeleteSessionStaleStmt("DELETE FROM session WHERE expires_at < now()", None)
}
impl DeleteSessionStaleStmt {
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
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[]).await
    }
}
pub struct CountRateLimitBucketStaleStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn count_rate_limit_bucket_stale() -> CountRateLimitBucketStaleStmt {
    CountRateLimitBucketStaleStmt(
        "SELECT count(*)::bigint AS n FROM rate_limit_bucket WHERE window_start < now() - interval '1 day'",
        None,
    )
}
impl CountRateLimitBucketStaleStmt {
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
    ) -> I64Query<'c, 'a, 's, C, i64, 0> {
        I64Query {
            client,
            params: [],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it,
        }
    }
}
pub struct DeleteRateLimitBucketStaleStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn delete_rate_limit_bucket_stale() -> DeleteRateLimitBucketStaleStmt {
    DeleteRateLimitBucketStaleStmt(
        "DELETE FROM rate_limit_bucket WHERE window_start < now() - interval '1 day'",
        None,
    )
}
impl DeleteRateLimitBucketStaleStmt {
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
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[]).await
    }
}
pub struct CountEmailLogOldStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn count_email_log_old() -> CountEmailLogOldStmt {
    CountEmailLogOldStmt(
        "SELECT count(*)::bigint AS n FROM email_log WHERE created_at < now() - make_interval(days => $1::int)",
        None,
    )
}
impl CountEmailLogOldStmt {
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
        days: &'a i32,
    ) -> I64Query<'c, 'a, 's, C, i64, 1> {
        I64Query {
            client,
            params: [days],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it,
        }
    }
}
pub struct DeleteEmailLogOldStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn delete_email_log_old() -> DeleteEmailLogOldStmt {
    DeleteEmailLogOldStmt(
        "DELETE FROM email_log WHERE created_at < now() - make_interval(days => $1::int)",
        None,
    )
}
impl DeleteEmailLogOldStmt {
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
        days: &'a i32,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[days]).await
    }
}
