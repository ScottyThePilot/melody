'use strict';
const CleverChannel = require('../modules/CleverChannel.js');

const channels = new Map();

async function getResponse(msg, channelID) {
  if (!channels.has(channelID)) channels.set(channelID, new CleverChannel(25));
  const channel = channels.get(channelID);
  return await channel.send(msg);
}

module.exports = {
  channels,
  getResponse
};
