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
    pub speech: bool,
}

#[derive(Clone)]
pub enum Message {
    Register((ActiveQuery, CancellationToken)),
    Deregister(ActiveQuery),
    Kill(Uuid),
    KillSpeech(Uuid),
}

pub async fn reaper(pool: Pool<AsyncMysqlConnection>, receiver: &mut Receiver<Message>) {
    let mut active_queries: HashMap<Uuid, HashMap<(i32, bool), CancellationToken>> = HashMap::new();
    
    while let Some(recv) = receiver.recv().await {
        match recv {
            Message::Register((aq, cancel_token)) => {
                if let Some(conn_id_map) = active_queries.get_mut(&aq.uuid) {
                    conn_id_map.insert((aq.conn_id, aq.speech), cancel_token);
                } else {
                    let mut conn_id_map = HashMap::new();
                    conn_id_map.insert((aq.conn_id, aq.speech), cancel_token);
                    active_queries.insert(aq.uuid, conn_id_map);
                }
            },
            Message::Deregister(aq) => {
                if let Some(conn_id_map) = active_queries.get_mut(&aq.uuid) {
                    conn_id_map.remove(&(aq.conn_id, aq.speech));
                }
            },
            Message::Kill(uuid) => {
                if let Some(conn_id_map) = active_queries.get_mut(&uuid) {
                    // cancel all tokens before killing active queries so the route handlers
                    // return the proper 204 code indicating a cancel, instead of an error
                    let conn_ids = conn_id_map
                        .iter()
                        .map(|((conn_id, _), cancel_token)| {cancel_token.cancel(); conn_id})
                        .collect::<Vec<&i32>>();
                    
                    if let Ok(mut conn) = pool.get().await {
                        for conn_id in conn_ids {
                            let _ = kill_connection_id(&mut conn, conn_id).await;
                        }
                    }
                    conn_id_map.clear();
                }
            },
            Message::KillSpeech(uuid) => {
                println!("killing only speeches");
                if let Some(conn_id_map) = active_queries.get_mut(&uuid) {
                    let conn_ids = conn_id_map
                        .iter()
                        .filter_map(|((conn_id, speech), cancel_token)| {
                            if *speech {
                                println!("cancelling {}", conn_id);
                                cancel_token.cancel(); 
                                Some(conn_id)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<&i32>>();
                    
                    if let Ok(mut conn) = pool.get().await {
                        for conn_id in conn_ids {
                            let _ = kill_connection_id(&mut conn, conn_id).await;
                            println!("killed {}", conn_id);
                        }
                    }
                    conn_id_map.clear();
                }
            }
        };
    }
}
