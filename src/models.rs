use super::schema::memes;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize)]
pub struct Meme {
    pub id: i32,
    pub name: String,
    pub image_path: String,
}

#[derive(Insertable, Serialize, Deserialize)]
#[table_name = "memes"]
pub struct NewMeme {
    pub name: String,
    pub image_path: String,
}
