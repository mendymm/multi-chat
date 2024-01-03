use chrono::SecondsFormat;
use log::info;
use redis::AsyncCommands;

use crate::types::ChatMsg;
use sqlx::{Executor, Pool, Sqlite, SqlitePool};
use std::sync::mpsc::Receiver;

pub async fn print_msgs(rx: Receiver<ChatMsg>) {
    info!("Starting printer thread");
    let client = redis::Client::open("redis://127.0.0.1").unwrap();
    let mut con = client.get_async_connection().await.unwrap();

    let opts = sqlx::sqlite::SqliteConnectOptions::default()
        .filename("tmp/messages.sqlite3")
        .create_if_missing(true);
    let conn = SqlitePool::connect_with(opts).await.unwrap();
    initialize_db(&conn).await;
    loop {
        // conn.ex
        let msg = rx.recv().unwrap();
        con.publish::<&str, String, i32>("chat", msg.to_html()).await.unwrap();
        msg.print_to_cli();
        let query: sqlx::query::Query<'_, Sqlite, _> =
            sqlx::query("insert into msgs(zstd_compress_msg_json,service,author,msg,published_at) values(?1,?2,?3,?4,?5)")
                .bind(msg.to_zstd_json())
                .bind(msg.location.name())
                .bind(msg.author)
                .bind(msg.msg_text)
                .bind(msg.published_at.to_rfc3339_opts(SecondsFormat::Micros,true));

        conn.execute(query).await.unwrap();
    }
}

async fn initialize_db(conn: &Pool<Sqlite>) {
    let query = r#"CREATE TABLE IF NOT EXISTS "msgs" (
        "id"	INTEGER NOT NULL UNIQUE,
        "service" TEXT,
        "author" TEXT,
        "msg" TEXT,
        "published_at" TEXT,
        "zstd_compress_msg_json"	BLOB,
        PRIMARY KEY("id" AUTOINCREMENT)
    );"#;
    conn.execute(query).await.unwrap();
}
