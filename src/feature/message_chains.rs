use ahash::AHasher;
use rand::Rng;
use serenity::model::id::{ChannelId, UserId};
use serenity::model::channel::Message;
use tokio::sync::Mutex;

use std::collections::HashMap;
use std::hash::{Hasher, Hash};
use std::sync::Arc;



pub type MessageChainsWrapper = Arc<Mutex<MessageChains>>;

impl From<MessageChains> for MessageChainsWrapper {
  fn from(message_chains: MessageChains) -> Self {
    Arc::new(Mutex::new(message_chains))
  }
}

#[derive(Debug, Clone, Default)]
pub struct MessageChains {
  chains: HashMap<ChannelId, MessageChain>
}

impl MessageChains {
  const ACTIVATION_THRESHOLD: usize = 3;
  const ACTIVATION_CHANCE: f64 = 0.375;

  pub fn new() -> MessageChains {
    Self::default()
  }

  pub fn should_contribute(&mut self, message: &Message) -> bool {
    let chain_len = self.observe_message(message);
    if chain_len >= Self::ACTIVATION_THRESHOLD && rand::thread_rng().gen_bool(Self::ACTIVATION_CHANCE) {
      self.reset_message_chain(message.channel_id);
      true
    } else {
      false
    }
  }

  fn observe_message(&mut self, message: &Message) -> usize {
    let message_content = message.content.trim();
    if message_content.is_empty() { return 0 };
    let previous = self.chains.get(&message.channel_id).copied();
    let chain = MessageChain::new(message_content, message.author.id, previous);
    self.chains.insert(message.channel_id, chain);
    chain.len
  }

  fn reset_message_chain(&mut self, channel: ChannelId) {
    self.chains.remove(&channel);
  }
}

#[derive(Debug, Clone, Copy)]
struct MessageChain {
  hash: u64,
  user: UserId,
  len: usize
}

impl MessageChain {
  fn new(content: &str, user: UserId, previous: Option<Self>) -> Self {
    let hash = get_content_hash(content);
    let len = match previous {
      Some(prev) if prev.continues(hash, user) => prev.len + 1,
      Some(..) | None => 1
    };

    MessageChain { hash, user, len }
  }

  /// Whether or not the next message continues the chain
  fn continues(self, hash: u64, user: UserId) -> bool {
    self.hash == hash && self.user != user
  }
}

fn get_content_hash(content: &str) -> u64 {
  let mut hasher = AHasher::default();
  content.trim().to_lowercase().hash(&mut hasher);
  hasher.finish()
}
