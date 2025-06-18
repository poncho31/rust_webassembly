use sqlx::{query, PgPool, Row};
use anyhow::{Error, Result};
use uuid::Uuid;
use time::OffsetDateTime;
use std::collections::HashMap;

pub struct DatabaseQuery {
    pool: PgPool,
}

impl DatabaseQuery {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_tables(&self, table_name: &str, columns: &str) -> Result<()> {
        let create_table_query = format!(
            "CREATE TABLE IF NOT EXISTS {} ({})",
            table_name, columns
        );
        
        sqlx::query(&create_table_query)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::msg(format!("Failed to create table {}: {}", table_name, e)))?;

        println!("Table {} created successfully", table_name);
        Ok(())
    }   

    pub async fn create_indexes(&self, table_name: &str, indexes: Vec<&str>) -> Result<()> {
        for idx in indexes {
            let index_query = format!(
                "CREATE INDEX IF NOT EXISTS idx_{}_{} ON {} ({})",
                table_name, idx, table_name, idx
            );
            sqlx::query(&index_query)
                .execute(&self.pool)
                .await
                .expect(&format!("Failed to create index for {}: {}", table_name, idx));
        }

        println!("Indexes created successfully for table: {}", table_name);
        Ok(())
    }

    
    /// Lance une requête en html brut => utilisé surtout dans la partie migration
    pub async fn run_query(&self, query: &str) -> Result<()> {
        // Lance la requête
        sqlx::query(query)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::msg(format!("Failed query {}", e)))?;

        println!("Query executed successfully: {}", query);
        Ok(())
    }
}
