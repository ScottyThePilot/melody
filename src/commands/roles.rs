use crate::prelude::*;
use crate::feature::roles::{Granter, JoinRoleFilter};
use crate::data::{Core, PersistGuild};
use crate::utils::RoleOrUser;
use super::{MelodyContext, CommandState};

use serenity::model::id::{UserId, GuildId, RoleId};
use serenity::model::guild::{Member, Role};
use serenity::utils::{content_safe, ContentSafeOptions};

use std::collections::{HashMap, HashSet};



const BOT_ROLE_TOO_LOW: &str = "The role you have specified is above my highest role and inaccessible to me";
const USER_ROLE_TOO_LOW: &str = "The role you have specified is above your highest role and not modifiable by you";
const MANAGED_ROLE: &str = "The role you have specified is a managed role and may not be used";

#[poise::command(
  slash_command,
  guild_only,
  subcommands(
    "role_grant",
    "role_revoke"
  ),
  name_localized("en-US", "role"),
  description_localized("en-US", "Allows a grantable role to be granted or revoked"),
  custom_data = CommandState::new()
    .usage_localized("en-US", [
      "/role grant <role> <user>",
      "/role revoke <role> <user>"
    ])
    .examples_localized("en-US", [
      "/role grant @Helper @Nanachi",
      "/role revoke @Helper @Reg"
    ])
)]
pub async fn role(_ctx: MelodyContext<'_>) -> MelodyResult {
  Err(MelodyError::command_precondition_violation("root command"))
}

#[poise::command(
  slash_command,
  guild_only,
  rename = "grant",
  name_localized("en-US", "grant"),
  description_localized("en-US", "Grants a grantable role to a user, as long as you are a valid granter"),
  custom_data = CommandState::new()
    .usage_localized("en-US", ["/role grant <role> <user>"])
    .examples_localized("en-US", ["/role grant @Helper @Nanachi"])
)]
async fn role_grant(
  ctx: MelodyContext<'_>,
  #[rename = "role"]
  #[description_localized("en-US", "The role to be granted")]
  role: Role,
  #[rename = "user"]
  #[description_localized("en-US", "The user to grant the role to")]
  user_id: UserId
) -> MelodyResult {
  let core = Core::from(ctx);
  let guild_id = ctx.guild_id().ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;
  let member1 = ctx.author_member().await.ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;

  let is_granter = core.operate_persist_guild(guild_id, |persist_guild| {
    Ok(is_granter(persist_guild, &member1, role.id))
  }).await?;

  let response = if is_granter {
    let member2 = guild_id.member(&core, user_id).await.context("failed to find member")?;
    if member2.add_role(&core, role.id).await.context("failed to add role").log_error().is_some() {
      info!(
        "User {user1} ({user1_id}) granted role {role} ({role_id}) to user {user2} ({user2_id}) in guild {guild_id}",
        role = role.name, role_id = role.id, guild_id = guild_id,
        user1 = member1.user.name, user1_id = member1.user.id,
        user2 = member2.user.name, user2_id = member2.user.id
      );

      format!("Granted role `@{}` to user `@{}`", role.name, member2.user.name)
    } else {
      format!("Failed to grant role `@{}` to user `@{}`", role.name, member2.user.name)
    }
  } else {
    "You cannot grant this role".to_owned()
  };

  ctx.reply(response).await.context("failed to send reply")?;
  Ok(())
}

#[poise::command(
  slash_command,
  guild_only,
  rename = "revoke",
  name_localized("en-US", "revoke"),
  description_localized("en-US", "Revokes a grantable role from a user, as long as you are a valid granter"),
  custom_data = CommandState::new()
    .usage_localized("en-US", ["/role revoke <role> <user>"])
    .examples_localized("en-US", ["/role revoke @Helper @Reg"])
)]
async fn role_revoke(
  ctx: MelodyContext<'_>,
  #[rename = "role"]
  #[description_localized("en-US", "The role to be revoked")]
  role: Role,
  #[rename = "user"]
  #[description_localized("en-US", "The user to revoke the role from")]
  user_id: UserId
) -> MelodyResult {
  let core = Core::from(ctx);
  let guild_id = ctx.guild_id().ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;
  let member1 = ctx.author_member().await.ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;

  let is_granter = core.operate_persist_guild(guild_id, |persist_guild| {
    Ok(is_granter(persist_guild, &member1, role.id))
  }).await?;

  let response = if is_granter {
    let member2 = guild_id.member(&core, user_id).await.context("failed to find member")?;
    if member2.remove_role(&core, role.id).await.context("failed to add role").log_error().is_some() {
      info!(
        "User {user1} ({user1_id}) revoked role {role} ({role_id}) from user {user2} ({user2_id}) in guild {guild_id}",
        role = role.name, role_id = role.id, guild_id = guild_id,
        user1 = member1.user.name, user1_id = member1.user.id,
        user2 = member2.user.name, user2_id = member2.user.id
      );

      format!("Revoked role `@{}` from user `@{}`", role.name, member2.user.name)
    } else {
      format!("Failed to revoke role `@{}` from user `@{}`", role.name, member2.user.name)
    }
  } else {
    "You cannot revoke this role".to_owned()
  };

  ctx.reply(response).await.context("failed to send reply")?;
  Ok(())
}

fn is_granter(persist_guild: &PersistGuild, member: &Member, role_id: RoleId) -> bool {
  persist_guild.grant_roles.get(&role_id).map_or(false, |granters| {
    granters.iter().any(|&granter| match granter {
      Granter::User(user_id) => member.user.id == user_id,
      Granter::Role(role_id) => member.roles.contains(&role_id)
    })
  })
}



#[poise::command(
  slash_command,
  guild_only,
  subcommands(
    "grant_roles_add",
    "grant_roles_remove",
    "grant_roles_add_granter",
    "grant_roles_remove_granter",
    "grant_roles_list"
  ),
  rename = "grant-roles",
  name_localized("en-US", "grant-roles"),
  description_localized("en-US", "Allows specific roles to be made grantable by specific users or other roles"),
  default_member_permissions = "MANAGE_ROLES",
  required_bot_permissions = "MANAGE_ROLES",
  required_permissions = "MANAGE_ROLES",
  custom_data = CommandState::new()
    .info_localized_concat("en-US", [
      "To create a grantable role, you must first register it with the `/grant-roles add` subcommand,",
      "and then add granters to it with the `/grant-roles add-granter` subcommand.",
      "Users will not be able to modify a grantable role if that role is above their highest role.",
      "The `/role grant` and `/role revoke` commands are used to grant and revoke grantable roles."
    ])
    .usage_localized("en-US", [
      "/grant-roles add <role>",
      "/grant-roles add-granter <role> <role|user>",
      "/grant-roles remove <role>",
      "/grant-roles remove-granter <role> <role|user>",
      "/grant-roles list [role]"
    ])
    .examples_localized("en-US", [
      "/grant-roles add @Helper",
      "/grant-roles add-granter @Helper @Mod",
      "/grant-roles add-granter @Helper @Admin",
      "/grant-roles add-granter @Helper @Nanachi",
      "/grant-roles remove @SuperAdmin",
      "/grant-roles remove-granter @Helper @Mod",
      "/grant-roles remove-granter @Helper @Admin",
      "/grant-roles remove-granter @Helper @Riko",
      "/grant-roles list",
      "/grant-roles list @Helper"
    ])
)]
pub async fn grant_roles(_ctx: MelodyContext<'_>) -> MelodyResult {
  Err(MelodyError::command_precondition_violation("root command"))
}

#[poise::command(
  slash_command,
  guild_only,
  rename = "add",
  name_localized("en-US", "add"),
  description_localized("en-US", "Adds a role that may be granted by a user group"),
  default_member_permissions = "MANAGE_ROLES",
  required_bot_permissions = "MANAGE_ROLES",
  required_permissions = "MANAGE_ROLES",
  custom_data = CommandState::new()
    .usage_localized("en-US", ["/grant-roles add <role>"])
    .examples_localized("en-US", ["/grant-roles add @Helper"])
)]
async fn grant_roles_add(
  ctx: MelodyContext<'_>,
  #[name_localized("en-US", "role")]
  #[description_localized("en-US", "The role to be added")]
  role: Role
) -> MelodyResult {
  roles_common(ctx, role, |persist_guild, _guild_id, role| {
    Ok(match persist_guild.grant_roles.insert(role.id, HashSet::new()) {
      Some(..) => format!("Reset granters for existing grant role `@{}`", role.name),
      None => format!("Created new grant role for `@{}` (Add granters with the `/grant-roles add-granter`)", role.name)
    })
  }).await
}

#[poise::command(
  slash_command,
  guild_only,
  rename = "remove",
  name_localized("en-US", "remove"),
  description_localized("en-US", "Removes a role from those that can be granted"),
  default_member_permissions = "MANAGE_ROLES",
  required_bot_permissions = "MANAGE_ROLES",
  required_permissions = "MANAGE_ROLES",
  custom_data = CommandState::new()
    .usage_localized("en-US", ["/grant-roles remove <role>"])
    .examples_localized("en-US", ["/grant-roles remove @SuperAdmin"])
)]
async fn grant_roles_remove(
  ctx: MelodyContext<'_>,
  #[name_localized("en-US", "role")]
  #[description_localized("en-US", "The role to be removed")]
  role: Role
) -> MelodyResult {
  roles_common(ctx, role, |persist_guild, _guild_id, role| {
    Ok(match persist_guild.grant_roles.remove(&role.id) {
      Some(..) => format!("Removed existing grant role `@{}`", role.name),
      None => format!("No grant role for `@{}` was found", role.name)
    })
  }).await
}

#[poise::command(
  slash_command,
  guild_only,
  rename = "add-granter",
  name_localized("en-US", "add-granter"),
  description_localized("en-US", "Adds a role or user to those allowed to grant a role"),
  default_member_permissions = "MANAGE_ROLES",
  required_bot_permissions = "MANAGE_ROLES",
  required_permissions = "MANAGE_ROLES",
  custom_data = CommandState::new()
    .usage_localized("en-US", [
      "/grant-roles add-granter <role> <role|user>"
    ])
    .examples_localized("en-US", [
      "/grant-roles add-granter @Helper @Mod",
      "/grant-roles add-granter @Helper @Admin",
      "/grant-roles add-granter @Helper @Nanachi",
    ])
)]
async fn grant_roles_add_granter(
  ctx: MelodyContext<'_>,
  #[name_localized("en-US", "role")]
  #[description_localized("en-US", "The role to be added")]
  role: Role,
  #[name_localized("en-US", "granter")]
  #[description_localized("en-US", "The role or user who should be able to grant this role")]
  granter: RoleOrUser
) -> MelodyResult {
  roles_common(ctx, role, |persist_guild, guild_id, role| {
    Ok(match persist_guild.grant_roles.get_mut(&role.id) {
      Some(granters) => {
        let granter = Granter::from(granter);
        let granter_description = granter.display(guild_id, ctx.cache());
        let granter_type = granter.type_str();

        match granters.insert(granter) {
          true => format!("Added {granter_description} as a granter for grantable role `@{}`", role.name),
          false => format!("That {granter_type} is already a granter for grantable role `@{}`", role.name)
        }
      },
      None => format!("The role `@{}` is not a grantable role", role.name)
    })
  }).await
}

#[poise::command(
  slash_command,
  guild_only,
  rename = "remove-granter",
  name_localized("en-US", "remove-granter"),
  description_localized("en-US", "Removes a role or user from those allowed to grant a role"),
  default_member_permissions = "MANAGE_ROLES",
  required_bot_permissions = "MANAGE_ROLES",
  required_permissions = "MANAGE_ROLES",
  custom_data = CommandState::new()
    .usage_localized("en-US", [
      "/grant-roles remove-granter <role> <role|user>"
    ])
    .examples_localized("en-US", [
      "/grant-roles remove-granter @Helper @Mod",
      "/grant-roles remove-granter @Helper @Admin",
      "/grant-roles remove-granter @Helper @Riko"
    ])
)]
async fn grant_roles_remove_granter(
  ctx: MelodyContext<'_>,
  #[name_localized("en-US", "role")]
  #[description_localized("en-US", "The role to be removed")]
  role: Role,
  #[name_localized("en-US", "granter")]
  #[description_localized("en-US", "The role or user who should no longer be able to grant this role")]
  granter: RoleOrUser
) -> MelodyResult {
  roles_common(ctx, role, |persist_guild, guild_id, role| {
    Ok(match persist_guild.grant_roles.get_mut(&role.id) {
      Some(granters) => {
        let granter = Granter::from(granter);
        let granter_description = granter.display(guild_id, ctx.cache());
        let granter_type = granter.type_str();

        match granters.remove(&granter) {
          true => format!("Removed {granter_description} as a granter for grantable role `@{}`", role.name),
          false => format!("That {granter_type} is not a granter for grantable role `@{}`", role.name)
        }
      },
      None => format!("The role `@{}` is not a grantable role", role.name)
    })
  }).await
}

#[poise::command(
  slash_command,
  guild_only,
  rename = "list",
  name_localized("en-US", "list"),
  description_localized("en-US", "Lists all grantable roles and their granters"),
  default_member_permissions = "MANAGE_ROLES",
  required_bot_permissions = "MANAGE_ROLES",
  required_permissions = "MANAGE_ROLES",
  custom_data = CommandState::new()
    .usage_localized("en-US", [
      "/grant-roles list [role]"
    ])
    .examples_localized("en-US", [
      "/grant-roles list",
      "/grant-roles list @Helper"
    ])
)]
async fn grant_roles_list(
  ctx: MelodyContext<'_>,
  #[name_localized("en-US", "role")]
  #[description_localized("en-US", "The role who's granters should be listed, if desired")]
  role: Option<Role>
) -> MelodyResult {
  let core = Core::from(ctx);
  let guild_id = ctx.guild_id().ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;

  let response = core.operate_persist_guild(guild_id, |persist_guild| {
    let content = if let Some(role) = role {
      if let Some(granters) = persist_guild.grant_roles.get(&role.id) {
        let granters_list = stringify_granters_list(&core, guild_id, granters);
        format!("The role `@{}` has the following granters:\n{granters_list}", role.name)
      } else {
        format!("The role `@{}` is not a grantable role", role.name)
      }
    } else {
      stringify_grant_roles(&core, guild_id, &persist_guild.grant_roles)
    };

    Ok(content_safe(&core, content, &ContentSafeOptions::new(), &[]))
  }).await?;

  ctx.reply(response).await.context("failed to send reply")?;
  Ok(())
}

fn stringify_grant_roles(core: &Core, guild_id: GuildId, grant_roles: &HashMap<RoleId, HashSet<Granter>>) -> String {
  if grant_roles.is_empty() {
    "There are no grantable roles registered".to_owned()
  } else {
    std::iter::once(format!("There are {} grantable roles:", grant_roles.len()))
      .chain(grant_roles.iter().map(|(&role_id, granters)| {
        let granters_list = stringify_granters_list(&core, guild_id, granters);
        let role_name = role_description(&core, guild_id, role_id);
        format!("- {role_name}: {granters_list}")
      }))
      .join("\n")
  }
}

fn stringify_granters_list(core: &Core, guild_id: GuildId, granters: &HashSet<Granter>) -> String {
  if granters.is_empty() {
    "none".to_owned()
  } else {
    granters.iter()
      .map(|&granter| match granter {
        Granter::Role(role_id) => role_description(core, guild_id, role_id),
        Granter::User(user_id) => user_description(core, user_id)
      })
      .join(", ")
  }
}

fn role_description(core: &Core, guild_id: GuildId, role_id: RoleId) -> String {
  #[allow(deprecated)]
  core.cache.role(guild_id, role_id)
    .map(|role| format!("role `@{}`", role.name))
    .unwrap_or_else(|| format!("role `{role_id}`"))
}

fn user_description(core: &Core, user_id: UserId) -> String {
  core.cache.user(user_id)
    .map(|user| format!("user `@{}`", user.name))
    .unwrap_or_else(|| format!("user `{user_id}`"))
}

#[poise::command(
  slash_command,
  guild_only,
  subcommands(
    "join_roles_add",
    "join_roles_remove",
    "join_roles_list"
  ),
  rename = "join-roles",
  name_localized("en-US", "join-roles"),
  description_localized("en-US", "Allows roles to be given to users or bots upon joining"),
  default_member_permissions = "MANAGE_ROLES",
  required_bot_permissions = "MANAGE_ROLES",
  required_permissions = "MANAGE_ROLES",
  custom_data = CommandState::new()
    .info_localized_concat("en-US", [
      "Users will not be able to modify a join role if that role is above their highest role."
    ])
    .usage_localized("en-US", [
      "/join-roles add <role> ['all'|'bots'|'humans']",
      "/join-roles remove <role> ['all'|'bots'|'humans']",
      "/join-roles list"
    ])
    .examples_localized("en-US", [
      "/join-roles add @Bots bots",
      "/join-roles add @Members humans",
      "/join-roles remove @Members",
      "/join-roles list"
    ])
)]
pub async fn join_roles(_ctx: MelodyContext<'_>) -> MelodyResult {
  Err(MelodyError::command_precondition_violation("root command"))
}

#[poise::command(
  slash_command,
  guild_only,
  rename = "add",
  name_localized("en-US", "add"),
  description_localized("en-US", "Adds a role to be given to users or bots on join"),
  default_member_permissions = "MANAGE_ROLES",
  required_bot_permissions = "MANAGE_ROLES",
  required_permissions = "MANAGE_ROLES",
  custom_data = CommandState::new()
    .usage_localized("en-US", [
      "/join-roles add <role> ['all'|'bots'|'humans']"
    ])
    .examples_localized("en-US", [
      "/join-roles add @Bots bots",
      "/join-roles add @Members humans"
    ])
)]
async fn join_roles_add(
  ctx: MelodyContext<'_>,
  #[name_localized("en-US", "role")]
  #[description_localized("en-US", "The role to be added to the join roles list")]
  role: Role,
  #[name_localized("en-US", "filter")]
  #[description_localized("en-US", "What types of accounts this role should be applied to (defaults to 'all')")]
  filter: Option<JoinRoleFilter>
) -> MelodyResult {
  let filter = filter.unwrap_or_default();
  roles_common(ctx, role, |persist_guild, _guild_id, role| {
    Ok(match persist_guild.join_roles.insert(role.id, filter) {
      Some(old_filter) => match filter == old_filter {
        true => format!("A join role already exists for `@{}` with filter `{}`", role.name, filter.to_str()),
        false => format!("Replaced existing join role for `@{}` with filter `{}`", role.name, filter.to_str())
      },
      None => format!("Created new join role for `@{}` with filter `{}`", role.name, filter.to_str())
    })
  }).await
}

#[poise::command(
  slash_command,
  guild_only,
  rename = "remove",
  name_localized("en-US", "remove"),
  description_localized("en-US", "Removes a role from those to be given to users or bots on join"),
  default_member_permissions = "MANAGE_ROLES",
  required_bot_permissions = "MANAGE_ROLES",
  required_permissions = "MANAGE_ROLES",
  custom_data = CommandState::new()
    .usage_localized("en-US", [
      "/join-roles remove <role> ['all'|'bots'|'humans']"
    ])
    .examples_localized("en-US", [
      "/join-roles remove @Members"
    ])
)]
async fn join_roles_remove(
  ctx: MelodyContext<'_>,
  #[name_localized("en-US", "role")]
  #[description_localized("en-US", "The role to be removed from the join roles list")]
  role: Role
) -> MelodyResult {
  roles_common(ctx, role, |persist_guild, _guild_id, role| {
    Ok(match persist_guild.join_roles.remove(&role.id) {
      Some(filter) => format!("Removed existing join role for `@{}` with filter `{}`", role.name, filter.to_str()),
      None => format!("No join role for `@{}` was found", role.name)
    })
  }).await
}

#[poise::command(
  slash_command,
  guild_only,
  rename = "list",
  name_localized("en-US", "list"),
  description_localized("en-US", "Lists all roles that can be given on join"),
  default_member_permissions = "MANAGE_ROLES",
  required_bot_permissions = "MANAGE_ROLES",
  required_permissions = "MANAGE_ROLES",
  custom_data = CommandState::new()
    .usage_localized("en-US", [
      "/join-roles list"
    ])
    .examples_localized("en-US", [
      "/join-roles list"
    ])
)]
async fn join_roles_list(ctx: MelodyContext<'_>) -> MelodyResult {
  let core = Core::from(ctx);
  let guild_id = ctx.guild_id().ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;
  let response = core.operate_persist_guild(guild_id, |persist_guild| {
    let content = stringify_join_roles(&core, guild_id, &persist_guild.join_roles);
    Ok(content_safe(&core, content, &ContentSafeOptions::new(), &[]))
  }).await?;

  ctx.reply(response).await.context("failed to send reply")?;
  Ok(())
}

// TODO: figure out how role positions actually work
fn member_role_position(member: &Member, core: &Core) -> Option<u16> {
  core.cache.guild(member.guild_id).map(|guild| {
    member.roles.iter()
      .filter_map(|role_id| guild.roles.get(role_id))
      .map(|role| role.position).max().unwrap_or(0)
  })
}

fn stringify_join_roles(core: &Core, guild_id: GuildId, join_roles: &HashMap<RoleId, JoinRoleFilter>) -> String {
  if join_roles.is_empty() {
    "There are no join roles registered".to_owned()
  } else {
    std::iter::once(format!("There are {} join roles:", join_roles.len()))
      .chain(join_roles.iter().map(|(&role_id, &filter)| {
        let role_name = role_description(core, guild_id, role_id);
        format!("- {role_name}: `{}`", filter.to_str())
      }))
      .join("\n")
  }
}



async fn roles_common<F>(ctx: MelodyContext<'_>, role: Role, operation: F) -> MelodyResult
where F: FnOnce(&mut crate::data::PersistGuild, GuildId, &Role) -> MelodyResult<String> {
  let core = Core::from(ctx);
  let guild_id = ctx.guild_id().ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;
  let member = ctx.author_member().await.ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;
  let user_role_position = member_role_position(&member, &core)
    .ok_or(MelodyError::command_cache_failure("guild"))?;
  let is_owner = is_owner(ctx, member.user.id)?;

  let response = if role.managed {
    MANAGED_ROLE.to_owned()
  } else if role.position >= user_role_position && !is_owner {
    USER_ROLE_TOO_LOW.to_owned()
  } else {
    let me = core.current_member(guild_id).await?;
    let my_role_position = member_role_position(&me, &core)
      .ok_or(MelodyError::command_cache_failure("guild"))?;

    let response = core.operate_persist_guild_commit(guild_id, |persist_guild| {
      operation(persist_guild, guild_id, &role).map(|content| {
        content_safe(&core, content, &ContentSafeOptions::new(), &[])
      })
    }).await?;

    let mut response = response;
    if role.position >= my_role_position {
      response.push('\n');
      response.push_str(BOT_ROLE_TOO_LOW);
    };

    response
  };

  ctx.reply(response).await.context("failed to send reply")?;
  Ok(())
}

fn is_owner(ctx: MelodyContext<'_>, user_id: UserId) -> MelodyResult<bool> {
  ctx.guild().map(|guild| guild.owner_id == user_id).ok_or(MelodyError::COMMAND_NOT_IN_GUILD)
}
