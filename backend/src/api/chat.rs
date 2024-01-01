use super::state::AppState;

use std::collections::HashMap;

use axum::{
    extract::{Query, State},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    Json,
};
use futures_util::stream::StreamExt;
use serde_json::json;
use tokio_stream::wrappers::BroadcastStream;

use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, QueryFilter};

use crate::entities::{
    chat::{ActiveModel as ActiveChat, Column, Entity as ChatEntity, Model as Chat},
    room::{ActiveModel as ActiveRoom, Entity as RoomEntity},
};

pub async fn subscribe(State(state): State<AppState>) -> impl IntoResponse {
    let stream = BroadcastStream::new(state.queue.subscribe()).map(|msg| match msg {
        Ok(msg) => Ok(Event::default()
            .event("message")
            .data(json!(msg).to_string())),
        Err(e) => Err(e),
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

#[derive(serde::Deserialize)]
pub struct NewMessage {
    pub sender: String,
    pub message: String,
    pub room_id: i32,
}

pub async fn send(
    State(state): State<AppState>,
    Json(new_message): Json<NewMessage>,
) -> Json<Chat> {
    let room = RoomEntity::find_by_id(new_message.room_id)
        .one(&state.conn)
        .await
        .unwrap()
        .unwrap();

    // parse string to vector
    let mut participants: Vec<String> = serde_json::from_str(&room.participants).unwrap();
    // if sender is not in participants, add them
    if !participants.contains(&new_message.sender) {
        participants.push(new_message.sender.clone());
    }

    // vector to string
    let participants = serde_json::to_string(&participants).unwrap();

    let room = ActiveRoom {
        id: ActiveValue::set(room.id),
        participants: ActiveValue::set(participants),
    };
    room.update(&state.conn)
        .await
        .expect("Error updating room participants");

    let new_message = ActiveChat {
        id: ActiveValue::not_set(),
        sender: ActiveValue::set(new_message.sender),
        message: ActiveValue::set(new_message.message),
        room_id: ActiveValue::set(new_message.room_id),
        timestamp: ActiveValue::set(chrono::Utc::now().to_rfc3339()),
    };

    let new_message = new_message
        .insert(&state.conn)
        .await
        .expect("Error inserting message");

    state
        .queue
        .send(new_message.clone())
        .expect("Error sending message");

    Json(new_message)
}

pub async fn get_chat(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<Vec<Chat>> {
    let room_id = params.get("room_id").unwrap();

    Json(
        ChatEntity::find()
            .filter(Column::RoomId.eq(room_id.parse::<i32>().unwrap()))
            .all(&state.conn)
            .await
            .unwrap(),
    )
}
