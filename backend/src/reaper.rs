use std::collections::HashMap;
use tokio::sync::mpsc::Receiver;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;
use crate::kill_connection_id;
use diesel_async::{pooled_connection::bb8::Pool, AsyncMysqlConnection};

#[derive(Clone)]
pub struct ActiveQuery {
    pub uuid: Uuid,
    pub conn_id: i32,
}

#[derive(Clone)]
pub enum Message {
    Register((ActiveQuery, CancellationToken)),
    Deregister(ActiveQuery),
    Kill(Uuid)
}

pub async fn reaper(pool: Pool<AsyncMysqlConnection>, receiver: &mut Receiver<Message>) {
    let mut active_queries: HashMap<Uuid, HashMap<i32, CancellationToken>> = HashMap::new();
    
    while let Some(recv) = receiver.recv().await {
        match recv {
            Message::Register((rq, cancel_token)) => {
                if let Some(conn_id_map) = active_queries.get_mut(&rq.uuid) {
                    conn_id_map.insert(rq.conn_id, cancel_token);
                } else {
                    let mut conn_id_map = HashMap::new();
                    conn_id_map.insert(rq.conn_id, cancel_token);
                    active_queries.insert(rq.uuid, conn_id_map);
                }
            },
            Message::Deregister(rq) => {
                if let Some(conn_id_map) = active_queries.get_mut(&rq.uuid) {
                    conn_id_map.remove(&rq.conn_id);
                }
            },
            Message::Kill(uuid) => {
                if let Some(conn_id_map) = active_queries.get_mut(&uuid) {
                    let conn_ids = conn_id_map
                        .iter()
                        .map(|(conn_id, cancel_token)| {cancel_token.cancel(); conn_id})
                        .collect::<Vec<&i32>>();
                    
                    if let Ok(mut conn) = pool.get().await {
                        for conn_id in conn_ids {
                            let _ = kill_connection_id(&mut conn, conn_id).await;
                        }
                    }
                    conn_id_map.clear();
                }
            },
        };
    }
}
