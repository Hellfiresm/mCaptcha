/*
 * Copyright (C) 2022  Aravinth Manivannan <realaravinth@batsense.net>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
use db_core::dev::*;
use std::str::FromStr;

use sqlx::postgres::PgPoolOptions;
use sqlx::types::time::OffsetDateTime;
use sqlx::PgPool;

pub mod errors;
#[cfg(test)]
pub mod tests;

#[derive(Clone)]
pub struct Database {
    pub pool: PgPool,
}

/// Use an existing database pool
pub struct Conn(pub PgPool);

/// Connect to databse
pub enum ConnectionOptions {
    /// fresh connection
    Fresh(Fresh),
    /// existing connection
    Existing(Conn),
}

pub struct Fresh {
    pub pool_options: PgPoolOptions,
    pub url: String,
}

pub mod dev {
    pub use super::errors::*;
    pub use super::Database;
    pub use db_core::dev::*;
    pub use prelude::*;
    pub use sqlx::Error;
}

pub mod prelude {
    pub use super::*;
    pub use db_core::prelude::*;
}

#[async_trait]
impl Connect for ConnectionOptions {
    type Pool = Database;
    async fn connect(self) -> DBResult<Self::Pool> {
        let pool = match self {
            Self::Fresh(fresh) => fresh
                .pool_options
                .connect(&fresh.url)
                .await
                .map_err(|e| DBError::DBError(Box::new(e)))?,
            Self::Existing(conn) => conn.0,
        };
        Ok(Database { pool })
    }
}

use dev::*;

#[async_trait]
impl Migrate for Database {
    async fn migrate(&self) -> DBResult<()> {
        sqlx::migrate!("./migrations/")
            .run(&self.pool)
            .await
            .map_err(|e| DBError::DBError(Box::new(e)))?;
        Ok(())
    }
}

#[async_trait]
impl MCDatabase for Database {
    /// ping DB
    async fn ping(&self) -> bool {
        use sqlx::Connection;

        if let Ok(mut con) = self.pool.acquire().await {
            con.ping().await.is_ok()
        } else {
            false
        }
    }

    /// register a new user
    async fn register(&self, p: &Register) -> DBResult<()> {
        let res;
        if let Some(email) = &p.email {
            res = sqlx::query!(
                "insert into mcaptcha_users 
        (name , password, email, secret) values ($1, $2, $3, $4)",
                &p.username,
                &p.hash,
                &email,
                &p.secret,
            )
            .execute(&self.pool)
            .await;
        } else {
            res = sqlx::query!(
                "INSERT INTO mcaptcha_users 
        (name , password,  secret) VALUES ($1, $2, $3)",
                &p.username,
                &p.hash,
                &p.secret,
            )
            .execute(&self.pool)
            .await;
        };
        res.map_err(map_register_err)?;
        Ok(())
    }

    /// delete a user
    async fn delete_user(&self, username: &str) -> DBResult<()> {
        sqlx::query!("DELETE FROM mcaptcha_users WHERE name = ($1)", username)
            .execute(&self.pool)
            .await
            .map_err(map_register_err)?;
        Ok(())
    }
}

fn now_unix_time_stamp() -> i64 {
    OffsetDateTime::now_utc().unix_timestamp()
}

//
//#[allow(non_snake_case)]
//struct InnerGistComment {
//    ID: i64,
//    owner: String,
//    comment: Option<String>,
//    gist_public_id: String,
//    created: i64,
//}
//
//impl From<InnerGistComment> for GistComment {
//    fn from(g: InnerGistComment) -> Self {
//        Self {
//            id: g.ID,
//            owner: g.owner,
//            comment: g.comment.unwrap(),
//            gist_public_id: g.gist_public_id,
//            created: g.created,
//        }
//    }
//}