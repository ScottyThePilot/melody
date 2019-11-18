'use strict';
const CleverChannel = require('../structures/CleverChannel.js');

class CleverBotAgent {
  constructor(historyLength) {
    this.channels = new Map();
    this.historyLength = historyLength;
  }

  async getResponse(msg, channelID) {
    if (!this.channels.has(channelID)) {
      this.channels.set(channelID, new CleverChannel(this.historyLength));
    }
    const channel = this.channels.get(channelID);
    return await channel.queue(msg);
  }

  get size() {
    return [...this.channels.values()].reduce((channel, a) => channel.size + a);
  }

  clearHistory() {
    this.channels.forEach((channel) => {
      channel.msgHistory = [];
    });
  }
}

module.exports = CleverBotAgent;
