'use strict';
const Logger = require('./Logger.js');
const GuildManager = require('./GuildManager.js');
const Command = require('./Command.js');
const Util = require('./util/Util.js');
const controller = {};

controller.destroyBot = async function destroyBot(client) {
  Logger.main.log('INFO', 'Shutting Down...');

  await Util.asyncForEach(client.guilds.array(), async (guild) => {
    await GuildManager.unload(guild.id);
    Logger.main.log('DATA', `Guild ${Logger.logifyGuild(guild)} unloaded`);
  });

  await Logger.main.end();
  
  await client.destroy();
}

controller.getAccessiblePlugins = async function getAccessiblePlugins(user, client) {
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

controller.onGuildMemberAdd = function onGuildMemberAdd(member, manager) {
  manager.log('LOGGER', `User ${Logger.logifyUser(member)} added to guild`);
}

controller.onGuildMemberRemove = function onGuildMemberRemove(member, manager) {
  manager.log('LOGGER', `User ${Logger.logifyUser(member)} removed from guild`);
}

controller.onMessageUpdate = function onMessageUpdate(oldMessage, newMessage, manager) {
  const oldContent = `Old Content: ${Logger.escape(Logger.cleanContent(oldMessage))}`;
  const oldMeta = Logger.stylizeMetaData(oldMessage).map((e) => '  ' + e);
  const newContent = `New Content: ${Logger.escape(Logger.cleanContent(newMessage))}`;
  const newMeta = Logger.stylizeMetaData(newMessage).map((e) => '  ' + e);
  manager.log('LOGGER', `Message by user ${Logger.logifyUser(oldMessage.author)} edited in channel ${Logger.logify(oldMessage.channel)}`, oldContent, ...oldMeta, newContent, ...newMeta);
}

controller.onMessageDelete = function onMessageDelete(message, manager) {
  const content = `Content: ${Logger.escape(Logger.cleanContent(message))}`;
  const meta = Logger.stylizeMetaData(message).map((e) => '  ' + e);
  manager.log('LOGGER', `Message by user ${Logger.logifyUser(message.author)} deleted in channel ${Logger.logify(message.channel)}`, content, ...meta);
}

controller.onMessageDeleteBulk = function onMessageDeleteBulk(messages, manager) {
  const list = messages.array().map((message) => {
    const header = `Message by user ${Logger.logifyUser(message.author)}:`;
    const content = `  Content: ${Logger.escape(Logger.cleanContent(message))}`;
    const meta = Logger.stylizeMetaData(message).map((e) => '    ' + e);
    return [header, content, ...meta];
  });
  manager.log('LOGGER', `Bulk message deletion in channel ${Logger.logify(messages.first().channel)}`, ...[].concat(...list));
}

controller.onMessage = function onMessage(message, manager) {
  const content = `Content: ${Logger.escape(Logger.cleanContent(message))}`;
  const meta = Logger.stylizeMetaData(message).map((e) => '  ' + e);
  manager.log('LOGGER', `Message by user ${Logger.logifyUser(message.author)} sent in channel ${Logger.logify(message.channel)}`, content, ...meta);
}

controller.userOwnsAGuild = function userOwnsAGuild(user, client) {
  return client.guilds.some((guild) => guild.owner.id === user.id);
}

controller.firstReady = false;

controller.wittyMuteMessages = [
  'I\'m afraid I can\'t let you do that. Send messages slower next time.',
  'Looks like you\'re sending messages a little too quickly.',
  'Please slow down, you\'re sending messages awful quickly.',
  'Please calm down, you\'re upsetting the robo-hampsters.',
  'Looks like you were sending messages too quickly.',
  'Try not to send messages so fast next time :(',
  'Whoa there! Slow down with the messages.',
  'Please don\'t send messages so quickly :(',
  'Next time, send messages a bit slower.',
  'Oh! So that\'s what that button does...',
  'It\'s rude to send messages so quickly.',
  'Enhance your calm.'
];

/*
const { scheduleJob } = require('node-schedule');

let job = scheduleJob('15 7 * * *', () => {

});
*/

controller.muteNoticeMessage = 'You were automatically muted for spamming. If you believe this is a bug, please contact this bot\'s owner, Scotty#4263';

module.exports = controller;
