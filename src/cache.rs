use tokio::sync::Mutex;
use sqlx::{Pool, Row, Sqlite};
use log::error;

pub struct Cache {
    character_names: Mutex<Vec<String>>,
}

impl Cache {
    pub fn new() -> Cache {
        Cache {
            character_names: Mutex::new(Vec::new()),
        }
    }

    pub async fn get_character_names(&self) -> Vec<String> {
        self.character_names.lock().await.clone()
    }

    pub async fn update_character_names(&self, db: &Pool<Sqlite>) {
        let entries = sqlx::query("SELECT name FROM characters")
            .fetch_all(db).await;

        if let Ok(entries) = entries {
            let mut names = self.character_names.lock().await;
            names.clear();
            names.extend(entries.iter().map(|x| x.get::<String, usize>(0)));
        } else {
            error!("Was unable to update character names!");
        }
    }
}
