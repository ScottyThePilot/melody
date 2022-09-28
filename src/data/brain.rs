use serenity::model::id::{ChannelId, UserId};
use serenity::model::channel::Message;
use tokio::sync::RwLock;

use std::collections::HashMap;
use std::sync::Arc;



pub type BrainContainer = Arc<RwLock<Brain>>;

#[derive(Debug, Clone, Default)]
pub struct Brain {
  pub chains: HashMap<ChannelId, BrainChain>
}

impl Brain {
  pub fn create() -> BrainContainer {
    Arc::new(RwLock::new(Brain::default()))
  }

  pub fn observe_message(&mut self, message: &Message) -> usize {
    if message.content.is_empty() { return 0 };
    let content = message.content.to_lowercase();
    let chain = self.chains.entry(message.channel_id)
      .and_modify(|chain| chain.advance(&content, message.author.id))
      .or_insert_with(|| BrainChain::new(&content, message.author.id));
    chain.len
  }

  pub fn reset_message_chain(&mut self, channel: ChannelId) {
    self.chains.remove(&channel);
  }
}

#[derive(Debug, Clone)]
pub struct BrainChain {
  content: String,
  user: UserId,
  len: usize
}

impl BrainChain {
  fn new(content: &str, user: UserId) -> Self {
    BrainChain { content: content.into(), user, len: 1 }
  }

  fn advance(&mut self, content: &str, user: UserId) {
    if self.content == content && self.user != user {
      self.user = user;
      self.len += 1;
    } else {
      self.content = content.to_owned();
      self.user = user;
      self.len = 0;
    }
  }
}
