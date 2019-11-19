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

module.exports = {
  onGuildMemberAdd,
  onGuildMemberRemove,
  onMessageUpdate,
  onMessageDelete,
  onMessageDeleteBulk,
  onMessage
};
