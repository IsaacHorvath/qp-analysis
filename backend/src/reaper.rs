use std::collections::HashMap;
use tokio::sync::mpsc::Receiver;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;
use crate::kill_connection_id;
use diesel_async::{pooled_connection::bb8::Pool, AsyncMysqlConnection};

/// A struct representing an active query that can be passed in registration and
/// deregistration messages.

#[derive(Clone)]
pub struct ActiveQuery {
    
    /// A unique user id, sent by the frontend.
    
    pub uuid: Uuid,
    
    /// An active MariaDB connection id, able to be killed.
    
    pub conn_id: i32,
    
    /// Whether this query is a speech query or not.
    
    pub speech: bool,
}

/// A message that can be sent by handlers and recieved by the reaper through a
/// tokio multi-producer single-consumer channel.

#[derive(Clone)]
pub enum Message {
    
    /// A registration message, including the query information and a cancellation
    /// token that can directly cancel the handler.
    ///
    /// This message means the handler is running an active query that should be
    /// killed if necessary.
    
    Register((ActiveQuery, CancellationToken)),
    
    /// A deregistration message, including the query information for reference.
    ///
    /// This message means the handler has finished running the query and it can be
    /// removed from the store.
    
    Deregister(ActiveQuery),
    
    /// A kill queries message, telling the reaper to kill all queries started by a
    /// particular user.
    
    Kill(Uuid),
    
    /// A kill speech queries message, telling the reaper to kill all speech
    /// queries started by a particular user.
    ///
    /// This method exists because the user might query speeches for a particular
    /// breakdown, and then change their mind and query another before the first
    /// finishes. In this case we want to cancel the existing speech query but not
    /// other ongoing chart queries that are still running in the main interface
    /// page.
    
    KillSpeech(Uuid),
}

/// An async reaper for the backend that kills database queries when requested.
///
/// This function works by holding a store of active queries and waiting for
/// messages. On receiving a registration message it stores the query details, and
/// on receiving a deregistration message these details are dropped. When a kill
/// message is received, the active queries associated with that user are killed
/// and their details dropped from the store.

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
                    // cancel all tokens *before* killing active queries so the route handlers
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
                if let Some(conn_id_map) = active_queries.get_mut(&uuid) {
                    let conn_ids = conn_id_map
                        .iter()
                        .filter_map(|((conn_id, speech), cancel_token)| {
                            if *speech {
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
                        }
                    }
                    conn_id_map.clear();
                }
            }
        };
    }
}
