use clap::Parser;
use pgerd::dot::{render_dot, FkRelation};
use sqlx::postgres::PgPoolOptions;
use sqlx::Row;

#[derive(Parser)]
#[command(
    name = "pgerd",
    about = "Renders a live Postgres database's real foreign-key relationships as a Graphviz DOT diagram"
)]
struct Cli {
    /// Postgres connection string.
    #[arg(long, env = "DATABASE_URL")]
    database_url: String,
    /// Schema to inspect.
    #[arg(long, default_value = "public")]
    schema: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let pool = PgPoolOptions::new().connect(&cli.database_url).await?;

    let table_rows =
        sqlx::query("SELECT tablename FROM pg_tables WHERE schemaname = $1 ORDER BY tablename")
            .bind(&cli.schema)
            .fetch_all(&pool)
            .await?;
    let tables: Vec<String> = table_rows
        .iter()
        .map(|r| r.get::<String, _>("tablename"))
        .collect();

    let fk_rows = sqlx::query(
        r#"
        SELECT
            c.conname AS constraint_name,
            c.conrelid::regclass::text AS from_table,
            c.confrelid::regclass::text AS to_table,
            (SELECT array_agg(a.attname ORDER BY k.ord)
             FROM unnest(c.conkey) WITH ORDINALITY AS k(attnum, ord)
             JOIN pg_attribute a ON a.attrelid = c.conrelid AND a.attnum = k.attnum) AS from_columns,
            (SELECT array_agg(a.attname ORDER BY k.ord)
             FROM unnest(c.confkey) WITH ORDINALITY AS k(attnum, ord)
             JOIN pg_attribute a ON a.attrelid = c.confrelid AND a.attnum = k.attnum) AS to_columns
        FROM pg_constraint c
        JOIN pg_namespace n ON n.oid = c.connamespace
        WHERE c.contype = 'f' AND n.nspname = $1
        ORDER BY from_table, constraint_name
        "#,
    )
    .bind(&cli.schema)
    .fetch_all(&pool)
    .await?;

    let relations: Vec<FkRelation> = fk_rows
        .iter()
        .map(|row| FkRelation {
            constraint_name: row.get("constraint_name"),
            from_table: row.get("from_table"),
            from_columns: row.get("from_columns"),
            to_table: row.get("to_table"),
            to_columns: row.get("to_columns"),
        })
        .collect();

    println!("{}", render_dot(&tables, &relations));
    Ok(())
}
