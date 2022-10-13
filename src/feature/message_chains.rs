use rand::Rng;
use serenity::model::id::{ChannelId, UserId};
use serenity::model::channel::Message;
use tokio::sync::Mutex;

use std::collections::HashMap;
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
  const ACTIVATION_CHANCE: f64 = 0.50;

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
    if message.content.is_empty() { return 0 };
    let content = message.content.to_lowercase();
    let chain = self.chains.entry(message.channel_id)
      .and_modify(|chain| chain.advance(&content, message.author.id))
      .or_insert_with(|| MessageChain::new(&content, message.author.id));
    chain.len
  }

  fn reset_message_chain(&mut self, channel: ChannelId) {
    self.chains.remove(&channel);
  }
}

#[derive(Debug, Clone)]
struct MessageChain {
  content: String,
  user: UserId,
  len: usize
}

impl MessageChain {
  fn new(content: &str, user: UserId) -> Self {
    MessageChain { content: content.into(), user, len: 1 }
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
