use sea_orm::DatabaseConnection;

use crate::entities::chat::Model as Chat;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState {
    pub conn: DatabaseConnection,
    pub queue: broadcast::Sender<Chat>,
}
