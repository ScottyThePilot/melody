pub struct Persist {}

#[derive(Serialize, Deserialize)]
pub struct PersistGuild {
  #[serde(default)]
  prefix: Option<String>
}
