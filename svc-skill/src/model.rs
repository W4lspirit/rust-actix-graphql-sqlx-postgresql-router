use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use async_graphql::dataloader::*;
use async_graphql::futures_util::{StreamExt, TryStreamExt};
use async_graphql::*;
use async_graphql::{Context, FieldResult, Object, SimpleObject};
use chrono::prelude::*;
use itertools::Itertools;
use sqlx::types::Uuid;
use sqlx::{FromRow, PgPool, query_as};

pub struct CoderDataLoader {
    pool: PgPool,
}

impl CoderDataLoader {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Loader<Uuid> for CoderDataLoader {
    type Value = Vec<Skill>;
    type Error = Arc<sqlx::Error>;
    async fn load(
        &self,
        keys: &[Uuid],
    ) -> std::result::Result<HashMap<Uuid, Self::Value>, Self::Error> {
        tracing::info!("Dataloading coders skills");
        let result1: Vec<(Uuid, Skill)> = sqlx::query_as!(Skill,"SELECT * FROM skill WHERE coder_id = ANY($1)", keys)
            .fetch(&self.pool)
            .map_ok(|skill: Skill| (skill.coder_id, skill))
            .map_err(|e| Arc::new(e))
            .try_collect().await?;
        let map = result1.into_iter().into_group_map();

        Ok(map)
    }
}

pub struct Coder {
    pub id: Uuid,
}

#[Object(extends)]
impl Coder {
    #[graphql(external)]
    async fn id(&self) -> &Uuid {
        &self.id
    }

    async fn skills<'a>(&self, ctx: &'a Context<'_>) -> FieldResult<Vec<Skill>> {
        tracing::info!("Loading coder skills");
        let loader = ctx.data_unchecked::<DataLoader<CoderDataLoader>>();
        let name: Option<Vec<Skill>> = loader.load_one(self.id).await?;
        Ok(name.unwrap_or_default())
    }
}

#[derive(SimpleObject, FromRow, Clone,Debug)]
pub struct Skill {
    pub id: Uuid,
    title: String,
    description: String,
    coder_id: Uuid,
    created_at: DateTime<Utc>,
}

impl Skill {
    pub async fn create(
        pool: &PgPool,
        coder_id: &str,
        title: &str,
        description: &str,
    ) -> Result<Skill> {
        let row = sqlx::query!(
            "INSERT INTO skill(title,description,coder_id) VALUES ($1,$2,$3) RETURNING id",
            title,
            description,
            Uuid::parse_str(coder_id)?
        )
        .fetch_one(pool)
        .await?;

        Ok(Skill {
            id: row.id,
            title: title.to_string(),
            description: description.to_string(),
            coder_id: Uuid::parse_str(coder_id)?,
            created_at: Utc::now(),
        })
    }

    pub async fn read_one(pool: &PgPool, id: &str) -> Result<Skill> {
        let row = sqlx::query_as!(
            Skill,
            "SELECT * FROM skill WHERE id = $1",
            Uuid::parse_str(id)?
        )
        .fetch_one(pool)
        .await?;

        Ok(row)
    }

    pub async fn read_by_coder(pool: &PgPool, coder_id: &Uuid) -> Result<Vec<Skill>> {
        let row = sqlx::query_as!(Skill, "SELECT * FROM skill WHERE coder_id = $1", coder_id)
            .fetch_all(pool)
            .await?;

        Ok(row)
    }

    pub async fn read_all(pool: &PgPool) -> Result<Vec<Skill>> {
        let rows = sqlx::query_as!(Skill, "SELECT * FROM skill")
            .fetch_all(pool)
            .await?;

        Ok(rows)
    }

    pub async fn update(pool: &PgPool, id: &str, description: &str) -> Result<Skill> {
        sqlx::query!(
            "UPDATE skill SET description=$1 WHERE id = $2",
            description,
            Uuid::parse_str(id)?
        )
        .execute(pool)
        .await?;

        Ok(Skill::read_one(pool, id).await?)
    }

    pub async fn delete(pool: &PgPool, id: &str) -> Result<()> {
        sqlx::query!("DELETE FROM skill WHERE id = $1", Uuid::parse_str(id)?)
            .execute(pool)
            .await?;

        Ok(())
    }
}
