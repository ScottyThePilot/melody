use crate::blueprint::RoleOrUser;

use serenity::all::GuildId;
use serenity::model::id::{UserId, RoleId};
use serenity::cache::Cache;

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum Granter {
  Role(RoleId),
  User(UserId)
}

impl Granter {
  pub fn type_str(self) -> &'static str {
    match self {
      Granter::Role(..) => "role",
      Granter::User(..) => "user"
    }
  }

  pub fn display(self, guild_id: GuildId, cache: &Cache) -> String {
    match self {
      Granter::Role(role_id) => DisplayRole::new(role_id, guild_id, cache).to_string(),
      Granter::User(user_id) => DisplayUser::new(user_id, cache).to_string()
    }
  }
}

impl From<RoleOrUser> for Granter {
  fn from(role_or_member: RoleOrUser) -> Self {
    match role_or_member {
      RoleOrUser::Role(role) => Granter::Role(role),
      RoleOrUser::User(user) => Granter::User(user)
    }
  }
}

#[derive(Debug, Clone)]
pub struct DisplayRole<'a> {
  role_id: RoleId,
  guild_id: GuildId,
  cache: &'a Cache
}

impl<'a> DisplayRole<'a> {
  pub fn new(role_id: RoleId, guild_id: GuildId, cache: &'a Cache) -> Self {
    DisplayRole { role_id, guild_id, cache }
  }
}

impl<'a> fmt::Display for DisplayRole<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let role = self.cache.role(self.guild_id, self.role_id);
    match role {
      Some(role) => write!(f, "role `@{}`", role.name),
      None => write!(f, "role `{}`", self.role_id)
    }
  }
}

#[derive(Debug, Clone)]
pub struct DisplayUser<'a> {
  user_id: UserId,
  cache: &'a Cache
}

impl<'a> DisplayUser<'a> {
  pub fn new(user_id: UserId, cache: &'a Cache) -> Self {
    DisplayUser { user_id, cache }
  }
}

impl<'a> fmt::Display for DisplayUser<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let user = self.cache.user(self.user_id);
    match user {
      Some(user) => write!(f, "user `@{}`", user.name),
      None => write!(f, "user `{}`", self.user_id)
    }
  }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum JoinRoleFilter {
  All, Bots, Humans
}

impl JoinRoleFilter {
  pub fn applies(self, is_bot: bool) -> bool {
    match self {
      Self::All => true,
      Self::Bots => is_bot,
      Self::Humans => !is_bot
    }
  }

  pub fn from_str(s: &str) -> Option<Self> {
    match s {
      "all" => Some(Self::All),
      "bots" => Some(Self::Bots),
      "humans" => Some(Self::Humans),
      _ => None
    }
  }

  pub fn to_str(self) -> &'static str {
    match self {
      Self::All => "all",
      Self::Bots => "bots",
      Self::Humans => "humans"
    }
  }
}
