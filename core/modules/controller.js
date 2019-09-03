'use strict';
const Logger = require('./Logger.js');
const GuildManager = require('./GuildManager.js');
const Command = require('./Command.js');
const Util = require('./util/Util.js');


async function destroyBot(client) {
  Logger.main.log('INFO', 'Shutting Down...');

  await Util.asyncForEach(client.guilds.array(), async (guild) => {
    await GuildManager.unload(guild.id);
    Logger.main.log('DATA', `Guild ${Logger.logifyGuild(guild)} unloaded`);
  });

  await Logger.main.end();
  
  await client.destroy();
}

async function getAccessiblePlugins(user, client) {
  let userPlugins = Command.pluginsDM.slice(0);

  await Util.asyncForEach([...GuildManager.all.values()], async (manager) => {
    let guild = client.guilds.get(manager.id);

    if (!guild.members.has(user.id)) return;

    let plugins = await manager.configdb.get('plugins');

    plugins.forEach((plugin) => {
      if (!userPlugins.includes(plugin)) userPlugins.push(plugin);
    });
  });

  return userPlugins;
}

function onGuildMemberAdd(member, manager) {
  manager.log('LOGGER', `User ${Logger.logifyUser(member)} added to guild`);
}

function onGuildMemberRemove(member, manager) {
  manager.log('LOGGER', `User ${Logger.logifyUser(member)} removed from guild`);
}

function onMessageUpdate(oldMessage, newMessage, manager) {
  const oldContent = `Old Content: ${Logger.escape(Logger.cleanContent(oldMessage))}`;
  const oldMeta = Logger.stylizeMetaData(oldMessage).map((e) => '  ' + e);
  const newContent = `New Content: ${Logger.escape(Logger.cleanContent(newMessage))}`;
  const newMeta = Logger.stylizeMetaData(newMessage).map((e) => '  ' + e);
  manager.log('LOGGER', `Message by user ${Logger.logifyUser(oldMessage.author)} edited in channel ${Logger.logify(oldMessage.channel)}`, oldContent, ...oldMeta, newContent, ...newMeta);
}

function onMessageDelete(message, manager) {
  const content = `Content: ${Logger.escape(Logger.cleanContent(message))}`;
  const meta = Logger.stylizeMetaData(message).map((e) => '  ' + e);
  manager.log('LOGGER', `Message by user ${Logger.logifyUser(message.author)} deleted in channel ${Logger.logify(message.channel)}`, content, ...meta);
}

function onMessageDeleteBulk(messages, manager) {
  const list = messages.array().map((message) => {
    const header = `Message by user ${Logger.logifyUser(message.author)}:`;
    const content = `  Content: ${Logger.escape(Logger.cleanContent(message))}`;
    const meta = Logger.stylizeMetaData(message).map((e) => '    ' + e);
    return [header, content, ...meta];
  });
  manager.log('LOGGER', `Bulk message deletion in channel ${Logger.logify(messages.first().channel)}`, ...[].concat(...list));
}

function onMessage(message, manager) {
  const content = `Content: ${Logger.escape(Logger.cleanContent(message))}`;
  const meta = Logger.stylizeMetaData(message).map((e) => '  ' + e);
  manager.log('LOGGER', `Message by user ${Logger.logifyUser(message.author)} sent in channel ${Logger.logify(message.channel)}`, content, ...meta);
}

function userOwnsAGuild(user, client) {
  return client.guilds.some((guild) => guild.owner.id === user.id);
}

function setup() {
  
}

/*
const { scheduleJob } = require('node-schedule');

let job = scheduleJob('15 7 * * *', () => {

});
*/

module.exports = {
  destroyBot,
  getAccessiblePlugins,
  onGuildMemberAdd,
  onGuildMemberRemove,
  onMessageUpdate,
  onMessageDelete,
  onMessageDeleteBulk,
  onMessage,
  userOwnsAGuild,
  setup,
  firstReady: false
};
