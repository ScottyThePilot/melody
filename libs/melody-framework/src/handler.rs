use serenity::client::FullEvent;
use serenity::gateway::ShardStageUpdateEvent;
use serenity::http::RatelimitInfo;
use serenity::model::prelude::*;

use std::collections::HashMap;



// https://docs.rs/serenity/0.12.4/src/serenity/client/event_handler.rs.html

macro_rules! handler {
  ($($VariantName:ident { $($arg:ident: $Arg:ty),* } => async fn $method:ident();)*) => (
    #[allow(unused_variables)]
    #[serenity::async_trait]
    pub trait MelodyHandler<S, E>: Send + Sync
    where S: Send + Sync, E: Send + Sync {
      $(async fn $method(&self, ctx: $crate::MelodyHandlerContext<'_, S, E>, $($arg: $Arg),*) {})*

      async fn command_error(&self, ctx: $crate::MelodyContext<'_, S, E>, error: $crate::MelodyFrameworkError<E>);
    }

    pub(crate) async fn dispatch<'a, S, E>(
      full_event: FullEvent,
      handler: &'a dyn MelodyHandler<S, E>,
      handler_context: $crate::MelodyHandlerContext<'a, S, E>
    ) where S: Send + Sync, E: Send + Sync {
      match full_event {
        $(FullEvent::$VariantName { $($arg),* } => handler.$method(handler_context, $($arg),*).await,)*
        _ => ()
      }
    }
  );
}

handler! {
  CommandPermissionsUpdate { permission: CommandPermissions } => async fn command_permissions_update();
  AutoModRuleCreate { rule: Rule } => async fn auto_moderation_rule_create();
  AutoModRuleUpdate { rule: Rule } => async fn auto_moderation_rule_update();
  AutoModRuleDelete { rule: Rule } => async fn auto_moderation_rule_delete();
  AutoModActionExecution { execution: ActionExecution } => async fn auto_moderation_action_execution();
  CacheReady { guilds: Vec<GuildId> } => async fn cache_ready();
  ShardsReady { total_shards: u32 } => async fn shards_ready();
  ChannelCreate { channel: GuildChannel } => async fn channel_create();
  CategoryCreate { category: GuildChannel } => async fn category_create();
  CategoryDelete { category: GuildChannel } => async fn category_delete();
  ChannelDelete { channel: GuildChannel, messages: Option<Vec<Message>> } => async fn channel_delete();
  ChannelPinsUpdate { pin: ChannelPinsUpdateEvent } => async fn channel_pins_update();
  ChannelUpdate { old: Option<GuildChannel>, new: GuildChannel } => async fn channel_update();
  GuildAuditLogEntryCreate { entry: AuditLogEntry, guild_id: GuildId } => async fn guild_audit_log_entry_create();
  GuildBanAddition { guild_id: GuildId, banned_user: User } => async fn guild_ban_addition();
  GuildBanRemoval { guild_id: GuildId, unbanned_user: User } => async fn guild_ban_removal();
  GuildCreate { guild: Guild, is_new: Option<bool> } => async fn guild_create();
  GuildDelete { incomplete: UnavailableGuild, full: Option<Guild> } => async fn guild_delete();
  GuildEmojisUpdate { guild_id: GuildId, current_state: HashMap<EmojiId, Emoji> } => async fn guild_emojis_update();
  GuildIntegrationsUpdate { guild_id: GuildId } => async fn guild_integrations_update();
  GuildMemberAddition { new_member: Member } => async fn guild_member_addition();
  GuildMemberRemoval { guild_id: GuildId, user: User, member_data_if_available: Option<Member> } => async fn guild_member_removal();
  GuildMemberUpdate { old_if_available: Option<Member>, new: Option<Member>, event: GuildMemberUpdateEvent } => async fn guild_member_update();
  GuildMembersChunk { chunk: GuildMembersChunkEvent } => async fn guild_members_chunk();
  GuildRoleCreate { new: Role } => async fn guild_role_create();
  GuildRoleDelete { guild_id: GuildId, removed_role_id: RoleId, removed_role_data_if_available: Option<Role> } => async fn guild_role_delete();
  GuildRoleUpdate { old_data_if_available: Option<Role>, new: Role } => async fn guild_role_update();
  GuildStickersUpdate { guild_id: GuildId, current_state: HashMap<StickerId, Sticker> } => async fn guild_stickers_update();
  GuildUpdate { old_data_if_available: Option<Guild>, new_data: PartialGuild } => async fn guild_update();
  InviteCreate { data: InviteCreateEvent } => async fn invite_create();
  InviteDelete { data: InviteDeleteEvent } => async fn invite_delete();
  Message { new_message: Message } => async fn message();
  MessageDelete { channel_id: ChannelId, deleted_message_id: MessageId, guild_id: Option<GuildId> } => async fn message_delete();
  MessageDeleteBulk { channel_id: ChannelId, multiple_deleted_messages_ids: Vec<MessageId>, guild_id: Option<GuildId> } => async fn message_delete_bulk();
  MessageUpdate { old_if_available: Option<Message>, new: Option<Message>, event: MessageUpdateEvent } => async fn message_update();
  ReactionAdd { add_reaction: Reaction } => async fn reaction_add();
  ReactionRemove { removed_reaction: Reaction } => async fn reaction_remove();
  ReactionRemoveAll { channel_id: ChannelId, removed_from_message_id: MessageId } => async fn reaction_remove_all();
  ReactionRemoveEmoji { removed_reactions: Reaction } => async fn reaction_remove_emoji();
  PresenceUpdate { new_data: Presence } => async fn presence_update();
  Ready { data_about_bot: Ready } => async fn ready();
  Resume { event: ResumedEvent } => async fn resume();
  ShardStageUpdate { event: ShardStageUpdateEvent } => async fn shard_stage_update();
  TypingStart { event: TypingStartEvent } => async fn typing_start();
  UserUpdate { old_data: Option<CurrentUser>, new: CurrentUser } => async fn user_update();
  VoiceServerUpdate { event: VoiceServerUpdateEvent } => async fn voice_server_update();
  VoiceStateUpdate { old: Option<VoiceState>, new: VoiceState } => async fn voice_state_update();
  VoiceChannelStatusUpdate { old: Option<String>, status: Option<String>, id: ChannelId, guild_id: GuildId } => async fn voice_channel_status_update();
  WebhookUpdate { guild_id: GuildId, belongs_to_channel_id: ChannelId } => async fn webhook_update();
  InteractionCreate { interaction: Interaction } => async fn interaction_create();
  IntegrationCreate { integration: Integration } => async fn integration_create();
  IntegrationUpdate { integration: Integration } => async fn integration_update();
  IntegrationDelete { integration_id: IntegrationId, guild_id: GuildId, application_id: Option<ApplicationId> } => async fn integration_delete();
  StageInstanceCreate { stage_instance: StageInstance } => async fn stage_instance_create();
  StageInstanceUpdate { stage_instance: StageInstance } => async fn stage_instance_update();
  StageInstanceDelete { stage_instance: StageInstance } => async fn stage_instance_delete();
  ThreadCreate { thread: GuildChannel } => async fn thread_create();
  ThreadUpdate { old: Option<GuildChannel>, new: GuildChannel } => async fn thread_update();
  ThreadDelete { thread: PartialGuildChannel, full_thread_data: Option<GuildChannel> } => async fn thread_delete();
  ThreadListSync { thread_list_sync: ThreadListSyncEvent } => async fn thread_list_sync();
  ThreadMemberUpdate { thread_member: ThreadMember } => async fn thread_member_update();
  ThreadMembersUpdate { thread_members_update: ThreadMembersUpdateEvent } => async fn thread_members_update();
  GuildScheduledEventCreate { event: ScheduledEvent } => async fn guild_scheduled_event_create();
  GuildScheduledEventUpdate { event: ScheduledEvent } => async fn guild_scheduled_event_update();
  GuildScheduledEventDelete { event: ScheduledEvent } => async fn guild_scheduled_event_delete();
  GuildScheduledEventUserAdd { subscribed: GuildScheduledEventUserAddEvent } => async fn guild_scheduled_event_user_add();
  GuildScheduledEventUserRemove { unsubscribed: GuildScheduledEventUserRemoveEvent } => async fn guild_scheduled_event_user_remove();
  EntitlementCreate { entitlement: Entitlement } => async fn entitlement_create();
  EntitlementUpdate { entitlement: Entitlement } => async fn entitlement_update();
  EntitlementDelete { entitlement: Entitlement } => async fn entitlement_delete();
  MessagePollVoteAdd { event: MessagePollVoteAddEvent } => async fn poll_vote_add();
  MessagePollVoteRemove { event: MessagePollVoteRemoveEvent } => async fn poll_vote_remove();
  Ratelimit { data: RatelimitInfo } => async fn ratelimit();
}
