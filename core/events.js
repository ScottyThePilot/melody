'use strict';
const util = require('./modules/util.js');

function onGuildMemberAdd(member, manager) {
  manager.log('LOGGER', `User ${util.logifyUser(member)} added to guild`);
}

function onGuildMemberRemove(member, manager) {
  manager.log('LOGGER', `User ${util.logifyUser(member)} removed from guild`);
}

function onMessageUpdate(oldMessage, newMessage, manager) {
  const oldContent = `Old Content: \"${util.escape(util.cleanContent(oldMessage))}\"`;
  const oldMeta = util.stylizeMetaData(oldMessage).map((e) => '  ' + e);
  const newContent = `New Content: \"${util.escape(util.cleanContent(newMessage))}\"`;
  const newMeta = util.stylizeMetaData(newMessage).map((e) => '  ' + e);
  manager.log('LOGGER', `Message by user ${util.logifyUser(oldMessage.author)} edited in channel ${util.logify(oldMessage.channel)}`, oldContent, ...oldMeta, newContent, ...newMeta);
}

function onMessageDelete(message, manager) {
  const content = `Content: \"${util.escape(util.cleanContent(message))}\"`;
  const meta = util.stylizeMetaData(message).map((e) => '  ' + e);
  manager.log('LOGGER', `Message by user ${util.logifyUser(message.author)} deleted in channel ${util.logify(message.channel)}`, content, ...meta);
}

function onMessageDeleteBulk(messages, manager) {
  const list = messages.array().map((message) => {
    const header = `Message by user ${util.logifyUser(message.author)}:`;
    const content = `  Content: \"${util.escape(util.cleanContent(message))}\"`;
    const meta = util.stylizeMetaData(message).map((e) => '    ' + e);
    return [header, content, ...meta];
  });
  manager.log('LOGGER', `Bulk message deletion in channel ${util.logify(messages.first().channel)}`, ...[].concat(...list));
}

function onMessage(message, manager) {
  const content = `Content: \"${util.escape(util.cleanContent(message))}\"`;
  const meta = util.stylizeMetaData(message).map((e) => '  ' + e);
  manager.log('LOGGER', `Message by user ${util.logifyUser(message.author)} sent in channel ${util.logify(message.channel)}`, content, ...meta);
}

async function onMessageNoCommand(melody, message) {
  const content = message.content.trim();
  const manager = message.guild ? melody.guildManagers.get(message.guild.id) : null;

  const isPing = melody.mention.test(content);
  const isZone = manager
    ? manager.configdb
        .getSync('cleverBotZones')
        .includes(message.channel.id)
    : false;

  // Send CleverBot response and exit if the match was a ping and that ping is the bot
  if (isPing || isZone) {
    let msg = isPing ? content.slice(content.match(melody.mention)[0].length).trim() : content;
    if (isPing && msg.startsWith(',')) msg = msg.slice(1).trim();

    const msgFailCatcher = util.makeCatcher(melody.logger, 'Unable to send message');

    message.channel.startTyping();

    const response = await melody.cleverBot.getResponse(msg, message.channel.id).catch((err) => {
      melody.log('WARN', 'Error while communicating with CleverBot API', err);
      return 'There was an error while communicating with the CleverBot API.';
    });

    message.channel.stopTyping();

    if (!response || !response.trim().length) return;

    await message.channel.send(`<@${message.author.id}>, ${response}`).catch(msgFailCatcher);
    return;
  }
}

module.exports = {
  onGuildMemberAdd,
  onGuildMemberRemove,
  onMessageUpdate,
  onMessageDelete,
  onMessageDeleteBulk,
  onMessage,
  onMessageNoCommand
};
