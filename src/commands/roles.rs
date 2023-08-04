use crate::{MelodyError, MelodyResult};
use crate::blueprint::*;
use crate::feature::roles::{Granter, JoinRoleFilter};
use crate::data::{Core, PersistGuild};
use crate::utils::Contextualize;

use itertools::Itertools;
use serenity::model::permissions::Permissions;
use serenity::model::id::{UserId, GuildId, RoleId};
use serenity::model::guild::{Member, Role, Guild};
use serenity::model::user::User;
use serenity::utils::{content_safe, ContentSafeOptions};

use std::collections::{HashMap, HashSet};



const BOT_MISSING_PERMS: &str = "I am missing `MANAGE_ROLES` permissions";
const BOT_ROLE_TOO_LOW: &str = "The role you have specified is above my highest role and inaccessible to me";
const USER_ROLE_TOO_LOW: &str = "The role you have specified is above your highest role and not modifiable by you";
const MANAGED_ROLE: &str = "The role you have specified is a managed role and may not be used";

pub const ROLE: BlueprintCommand = blueprint_command! {
  name: "role",
  description: "Allows a grantable role to be granted or revoked",
  usage: [
    "/role grant <role> <user>",
    "/role revoke <role> <user>"
  ],
  examples: [
    "/role grant @Helper @Nanachi",
    "/role revoke @Helper @Reg"
  ],
  allow_in_dms: false,
  subcommands: [
    blueprint_subcommand! {
      name: "grant",
      description: "Grants a grantable role to a user, as long as you are a valid granter",
      arguments: [
        blueprint_argument!(Role {
          name: "role",
          description: "The role to be granted",
          required: true
        }),
        blueprint_argument!(User {
          name: "user",
          description: "The user to grant the role to"
        })
      ],
      function: role_grant
    },
    blueprint_subcommand! {
      name: "revoke",
      description: "Revokes a grantable role from a user, as long as you are a valid granter",
      arguments: [
        blueprint_argument!(Role {
          name: "role",
          description: "The role to be revoked",
          required: true
        }),
        blueprint_argument!(User {
          name: "user",
          description: "The user to revoke the role from"
        })
      ],
      function: role_revoke
    }
  ]
};

#[command_attr::hook]
async fn role_grant(core: Core, args: BlueprintCommandArgs) -> MelodyResult {
  let guild_id = args.interaction.guild_id.ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;
  let member = args.interaction.member.clone().ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;
  let (role, user) = resolve_arguments::<(Role, User)>(args.option_values)?;
  let is_granter = core.operate_persist_guild(guild_id, |persist_guild| {
    Ok(is_granter(persist_guild, &member, role.id))
  }).await?;

  let response = if is_granter {
    let mut member = guild_id.member(&core, user.id).await.context("failed to find member")?;
    if member.add_role(&core, role.id).await.context_log("failed to add role") {
      format!("Granted role `@{}` to user `@{}`", role.name, user.name)
    } else {
      format!("Failed to grant role `@{}` to user `@{}`", role.name, user.name)
    }
  } else {
    "You cannot grant this role".to_owned()
  };

  BlueprintCommandResponse::new(response)
    .send(&core, &args.interaction).await
}

#[command_attr::hook]
async fn role_revoke(core: Core, args: BlueprintCommandArgs) -> MelodyResult {
  let guild_id = args.interaction.guild_id.ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;
  let member = args.interaction.member.clone().ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;
  let (role, user) = resolve_arguments::<(Role, User)>(args.option_values)?;
  let is_granter = core.operate_persist_guild(guild_id, |persist_guild| {
    Ok(is_granter(persist_guild, &member, role.id))
  }).await?;

  let response = if is_granter {
    let mut member = guild_id.member(&core, user.id).await.context("failed to find member")?;
    if member.remove_role(&core, role.id).await.context_log("failed to add role") {
      format!("Revoked role `@{}` from user `@{}`", role.name, user.name)
    } else {
      format!("Failed to revoke role `@{}` from user `@{}`", role.name, user.name)
    }
  } else {
    "You cannot revoke this role".to_owned()
  };

  BlueprintCommandResponse::new(response)
    .send(&core, &args.interaction).await
}

pub const GRANT_ROLES: BlueprintCommand = blueprint_command! {
  name: "grant-roles",
  description: "Allows specific roles to be made grantable by specific users or other roles",
  usage: [
    "/grant-roles add <role>",
    "/grant-roles add-granter <role> <role|user>",
    "/grant-roles remove <role>",
    "/grant-roles remove-granter <role> <role|user>",
    "/grant-roles list [role]"
  ],
  examples: [
    "/grant-roles add @Helper",
    "/grant-roles add-granter @Helper @Mod",
    "/grant-roles add-granter @Helper @Admin",
    "/grant-roles add-granter @Helper @Nanachi",
    "/grant-roles remove @SuperAdmin",
    "/grant-roles list",
    "/grant-roles list @Helper"
  ],
  allow_in_dms: false,
  default_permissions: Permissions::MANAGE_ROLES,
  subcommands: [
    blueprint_subcommand! {
      name: "add",
      description: "Adds a role that may be granted by a user group",
      arguments: [
        blueprint_argument!(Role {
          name: "role",
          description: "The role to be added",
          required: true
        })
      ],
      function: grant_roles_add
    },
    blueprint_subcommand! {
      name: "remove",
      description: "Removes a role from those that can be granted",
      arguments: [
        blueprint_argument!(Role {
          name: "role",
          description: "The role to be removed",
          required: true
        })
      ],
      function: grant_roles_remove
    },
    blueprint_subcommand! {
      name: "add-granter",
      description: "Adds a role or user to those allowed to grant a role",
      arguments: [
        blueprint_argument!(Role {
          name: "role",
          description: "The role to be added",
          required: true
        }),
        blueprint_argument!(Mentionable {
          name: "granter",
          description: "The role or user who should be able to grant this role",
          required: true
        })
      ],
      function: grant_roles_add_granter
    },
    blueprint_subcommand! {
      name: "remove-granter",
      description: "Removes a role or user from those allowed to grant a role",
      arguments: [
        blueprint_argument!(Role {
          name: "role",
          description: "The role to be removed",
          required: true
        }),
        blueprint_argument!(Mentionable {
          name: "granter",
          description: "The role or user who should no longer be able to grant this role",
          required: true
        })
      ],
      function: grant_roles_remove_granter
    },
    blueprint_subcommand! {
      name: "list",
      description: "Lists all grantable roles and their granters",
      arguments: [
        blueprint_argument!(Role {
          name: "role",
          description: "The role who's granters should be listed"
        })
      ],
      function: grant_roles_list
    }
  ]
};

#[command_attr::hook]
async fn grant_roles_add(core: Core, args: BlueprintCommandArgs) -> MelodyResult {
  grant_roles_add_remove(core, args, |persist_guild, role, ()| {
    Ok(match persist_guild.grant_roles.insert(role.id, HashSet::new()) {
      Some(..) => format!("Reset granters for existing grant role `@{}`", role.name),
      None => format!("Created new grant role for `@{}` (Add granters with the `/grant-roles add-granter`)", role.name)
    })
  }).await
}

#[command_attr::hook]
async fn grant_roles_remove(core: Core, args: BlueprintCommandArgs) -> MelodyResult {
  grant_roles_add_remove(core, args, |persist_guild, role, ()| {
    Ok(match persist_guild.grant_roles.remove(&role.id) {
      Some(..) => format!("Removed existing grant role `@{}`", role.name),
      None => format!("No grant role for `@{}` was found", role.name)
    })
  }).await
}

#[command_attr::hook]
async fn grant_roles_add_granter(core: Core, args: BlueprintCommandArgs) -> MelodyResult {
  grant_roles_add_remove_granter(core, args, |persist_guild, role, granter| {
    Ok(match persist_guild.grant_roles.get_mut(&role.id) {
      Some(granters) => {
        let (granter, granter_description, granter_type) = match &granter {
          RoleOrMember::Member(user, ..) => (Granter::User(user.id), format!("user `@{}`", user.name), "user"),
          RoleOrMember::Role(role) => (Granter::Role(role.id), format!("role `@{}`", role.name), "role")
        };

        match granters.insert(granter) {
          true => format!("Added {granter_description} as a granter for grantable role `@{}`", role.name),
          false => format!("That {granter_type} is already a granter for grantable role `@{}`", role.name)
        }
      },
      None => format!("The role `@{}` is not a grantable role", role.name)
    })
  }).await
}

#[command_attr::hook]
async fn grant_roles_remove_granter(core: Core, args: BlueprintCommandArgs) -> MelodyResult {
  grant_roles_add_remove_granter(core, args, |persist_guild, role, granter| {
    Ok(match persist_guild.grant_roles.get_mut(&role.id) {
      Some(granters) => {
        let (granter, granter_description, granter_type) = match &granter {
          RoleOrMember::Member(user, ..) => (Granter::User(user.id), format!("user `@{}`", user.name), "user"),
          RoleOrMember::Role(role) => (Granter::Role(role.id), format!("role `@{}`", role.name), "role")
        };

        match granters.remove(&granter) {
          true => format!("Removed {granter_description} as a granter for grantable role `@{}`", role.name),
          false => format!("That {granter_type} is not a granter for grantable role `@{}`", role.name)
        }
      },
      None => format!("The role `@{}` is not a grantable role", role.name)
    })
  }).await
}

#[command_attr::hook]
async fn grant_roles_list(core: Core, args: BlueprintCommandArgs) -> MelodyResult {
  let guild_id = args.interaction.guild_id.ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;
  let role = resolve_arguments::<Option<Role>>(args.option_values)?;
  let response = core.operate_persist_guild(guild_id, |persist_guild| {
    let content = if let Some(role) = role {
      if let Some(granters) = persist_guild.grant_roles.get(&role.id) {
        let granters_list = stringify_granters_list(&core, guild_id, granters);
        format!("The role `@{}` has the following granters:\n{granters_list}", role.name)
      } else {
        format!("The role `@{}` is not a grantable", role.name)
      }
    } else {
      stringify_grant_roles(&core, guild_id, &persist_guild.grant_roles)
    };

    Ok(content_safe(&core, content, &ContentSafeOptions::new(), &[]))
  }).await?;

  BlueprintCommandResponse::new(response)
    .send(&core, &args.interaction).await
}

async fn grant_roles_add_remove<F>(core: Core, args: BlueprintCommandArgs, operation: F) -> MelodyResult
where F: FnOnce(&mut crate::data::PersistGuild, &Role, ()) -> MelodyResult<String> {
  roles_common(core, args, |option_values| {
    resolve_arguments::<Role>(option_values).map(|role| (role, ()))
  }, operation).await
}

async fn grant_roles_add_remove_granter<F>(core: Core, args: BlueprintCommandArgs, operation: F) -> MelodyResult
where F: FnOnce(&mut crate::data::PersistGuild, &Role, RoleOrMember) -> MelodyResult<String> {
  roles_common(core, args, |option_values| {
    resolve_arguments::<(Role, RoleOrMember)>(option_values)
  }, operation).await
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
  core.cache.role(guild_id, role_id)
    .map(|role| format!("role `@{}`", role.name))
    .unwrap_or_else(|| format!("role `{role_id}`"))
}

fn user_description(core: &Core, user_id: UserId) -> String {
  core.cache.user(user_id)
    .map(|user| format!("user `@{}`", user.name))
    .unwrap_or_else(|| format!("user `{user_id}`"))
}

pub const JOIN_ROLES: BlueprintCommand = blueprint_command! {
  name: "join-roles",
  description: "Allows roles to be given to users or bots upon joining",
  usage: [
    "/join-roles add <role> ['all'|'bots'|'humans']",
    "/join-roles remove <role> ['all'|'bots'|'humans']",
    "/join-roles list"
  ],
  examples: [
    "/join-roles add @Bots bots",
    "/join-roles add @Members humans",
    "/join-roles remove @Members",
    "/join-roles list"
  ],
  allow_in_dms: false,
  default_permissions: Permissions::MANAGE_ROLES,
  subcommands: [
    blueprint_subcommand! {
      name: "add",
      description: "Adds a role to be given to users or bots on join",
      arguments: [
        blueprint_argument!(Role {
          name: "role",
          description: "The role to be added",
          required: true
        }),
        FILTER_ARGUMENT
      ],
      function: join_roles_add
    },
    blueprint_subcommand! {
      name: "remove",
      description: "Removes a role from those to be given to users or bots on join",
      arguments: [
        blueprint_argument!(Role {
          name: "role",
          description: "The role to be removed",
          required: true
        }),
        FILTER_ARGUMENT
      ],
      function: join_roles_remove
    },
    blueprint_subcommand! {
      name: "list",
      description: "Lists all roles that will be given on join",
      arguments: [],
      function: join_roles_list
    }
  ]
};

#[command_attr::hook]
async fn join_roles_add(core: Core, args: BlueprintCommandArgs) -> MelodyResult {
  join_roles_add_remove(core, args, |persist_guild, role, filter| {
    Ok(match persist_guild.join_roles.insert(role.id, filter) {
      Some(..) => format!("Replaced existing join role filter for `@{}` with filter `{}`", role.name, filter.to_str()),
      None => format!("Created new join role for `@{}` with filter `{}`", role.name, filter.to_str())
    })
  }).await
}

#[command_attr::hook]
async fn join_roles_remove(core: Core, args: BlueprintCommandArgs) -> MelodyResult {
  join_roles_add_remove(core, args, |persist_guild, role, filter| {
    Ok(match persist_guild.join_roles.remove(&role.id) {
      Some(..) => format!("Removed existing join role for `@{}` with filter `{}`", role.name, filter.to_str()),
      None => format!("No join role for `@{}` was found", role.name)
    })
  }).await
}

#[command_attr::hook]
async fn join_roles_list(core: Core, args: BlueprintCommandArgs) -> MelodyResult {
  let guild_id = args.interaction.guild_id.ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;
  let response = core.operate_persist_guild(guild_id, |persist_guild| {
    let content = stringify_join_roles(&core, guild_id, &persist_guild.join_roles);
    Ok(content_safe(&core, content, &ContentSafeOptions::new(), &[]))
  }).await?;

  BlueprintCommandResponse::new(response)
    .send(&core, &args.interaction).await
}

async fn join_roles_add_remove<F>(core: Core, args: BlueprintCommandArgs, operation: F) -> MelodyResult
where F: FnOnce(&mut crate::data::PersistGuild, &Role, JoinRoleFilter) -> MelodyResult<String> {
  roles_common(core, args, |option_values| {
    let (role, filter) = resolve_arguments::<(Role, Option<String>)>(option_values)?;
    let filter = filter.as_deref().map_or(Some(JoinRoleFilter::All), JoinRoleFilter::from_str)
      .ok_or(MelodyError::COMMAND_INVALID_ARGUMENTS_STRUCTURE)?;
    Ok((role, filter))
  }, operation).await
}

fn member_role_position(member: &Member, core: &Core) -> Option<i64> {
  core.cache.guild_field(member.guild_id, |guild| {
    member.roles.iter()
      .filter_map(|role_id| guild.roles.get(role_id))
      .map(|role| role.position).max().unwrap_or(-1)
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

const FILTER_ARGUMENT: BlueprintOption = blueprint_argument!(String {
  name: "filter",
  description: "What types of accounts this role should be applied to (defaults to 'all')",
  choices: [
    ("all", "all"),
    ("bots", "bots"),
    ("humans", "humans")
  ]
});

fn is_granter(persist_guild: &PersistGuild, member: &Member, role_id: RoleId) -> bool {
  persist_guild.grant_roles.get(&role_id).map_or(false, |granters| {
    granters.iter().any(|&granter| match granter {
      Granter::User(user_id) => member.user.id == user_id,
      Granter::Role(role_id) => member.roles.contains(&role_id)
    })
  })
}



async fn roles_common<F, FA, A>(
  core: Core, args: BlueprintCommandArgs,
  args_operation: FA, operation: F
) -> MelodyResult where
  F: FnOnce(&mut crate::data::PersistGuild, &Role, A) -> MelodyResult<String>,
  FA: FnOnce(CommandDataOptionValues) -> MelodyResult<(Role, A)>
{
  let guild_id = args.interaction.guild_id.ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;
  let (role, args_remaining) = args_operation(args.option_values)?;

  let member = args.interaction.member.clone().ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;
  let user_role_position = member_role_position(&member, &core)
    .ok_or(MelodyError::command_cache_failure("guild"))?;
  let is_owner = core.cache.guild_field(guild_id, |guild| guild.owner_id == member.user.id)
    .ok_or(MelodyError::command_cache_failure("guild"))?;

  let response = if role.managed {
    MANAGED_ROLE.to_owned()
  } else if role.position >= user_role_position && !is_owner {
    USER_ROLE_TOO_LOW.to_owned()
  } else {
    let me = core.current_member(guild_id).await?;
    let my_role_position = member_role_position(&me, &core)
      .ok_or(MelodyError::command_cache_failure("guild"))?;
    let permissions = me.permissions(&core).context("failed to get permissions for member")?;

    let response = core.operate_persist_guild_commit(guild_id, |persist_guild| {
      operation(persist_guild, &role, args_remaining).map(|content| {
        content_safe(&core, content, &ContentSafeOptions::new(), &[])
      })
    }).await?;

    std::iter::once(response)
      .chain((!permissions.manage_roles()).then(|| BOT_MISSING_PERMS.to_owned()))
      .chain((role.position >= my_role_position).then(|| BOT_ROLE_TOO_LOW.to_owned()))
      .join("\n")
  };

  BlueprintCommandResponse::new(response)
    .send(&core, &args.interaction).await
}
