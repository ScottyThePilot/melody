use crate::blueprint::RoleOrMember;

use serenity::model::id::{UserId, RoleId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum Granter {
  Role(RoleId),
  User(UserId)
}

impl Granter {
  pub fn from_role_or_member(role_or_member: &RoleOrMember) -> Self {
    match role_or_member {
      RoleOrMember::Role(role) => Granter::Role(role.id),
      RoleOrMember::Member(user, ..) => Granter::User(user.id)
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
