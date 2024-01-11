use ahash::AHasher;
use chrono::{DateTime, Utc};
use serenity::model::id::{EmojiId, UserId};
use serenity::model::guild::Emoji;

use std::collections::HashMap;
use std::hash::{Hasher, Hash};



#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(transparent)]
pub struct EmojiStats {
  uses: HashMap<EmojiId, usize>,
  #[serde(skip, default)]
  last_interaction: Option<EmojiInteraction>,
}

impl EmojiStats {
  pub fn increment_emoji_uses(&mut self, emoji_id: EmojiId, user_id: UserId) {
    if self.try_interact(emoji_id, user_id, true) {
      increment(self.uses.entry(emoji_id).or_default());
    };
  }

  pub fn decrement_emoji_uses(&mut self, emoji_id: EmojiId, user_id: UserId) {
    if self.try_interact(emoji_id, user_id, false) {
      self.uses.get_mut(&emoji_id).map(decrement);
    };
  }

  pub fn get_emoji_uses<'a, F>(&self, mut f: F) -> Vec<(Emoji, usize)>
  where F: FnMut(EmojiId) -> Option<&'a Emoji> {
    let mut uses = self.uses.iter()
      .filter_map(|(&id, &c)| (c > 0).then_some((id, c)))
      .filter_map(|(id, c)| f(id).map(|emoji| (emoji.clone(), c)))
      .collect::<Vec<(Emoji, usize)>>();
    uses.sort_unstable_by(|a, b| Ord::cmp(&a.1, &b.1).reverse());
    uses
  }

  fn try_interact(&mut self, emoji_id: EmojiId, user_id: UserId, state: bool) -> bool {
    let now = Utc::now();
    let hash = hash_interaction(emoji_id, user_id, state);
    match self.last_interaction.replace(EmojiInteraction { hash, time: now }) {
      Some(interaction) => interaction.is_valid(hash, now),
      None => true
    }
  }
}

#[derive(Debug, Clone, Copy)]
struct EmojiInteraction {
  hash: u64,
  time: DateTime<Utc>
}

impl EmojiInteraction {
  fn is_valid(self, hash: u64, now: DateTime<Utc>) -> bool {
    self.hash != hash || now.signed_duration_since(self.time).num_seconds() > 5
  }
}

fn hash_interaction(emoji_id: EmojiId, user_id: UserId, state: bool) -> u64 {
  let mut hasher = AHasher::default();
  emoji_id.hash(&mut hasher);
  user_id.hash(&mut hasher);
  state.hash(&mut hasher);
  hasher.finish()
}

fn increment(i: &mut usize) {
  *i = i.saturating_add(1);
}

fn decrement(i: &mut usize) {
  *i = i.saturating_sub(1);
}
