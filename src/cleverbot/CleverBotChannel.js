'use strict';
const CleverBot = require('./CleverBot.js');
const Queue = require('../core/Queue.js');

class CleverBotChannel {
  constructor(len) {
    this.bot = new CleverBot(len);
    this.queue = new Queue();
  }

  send(msg) {
    return this.queue.pushPromise(() => this.bot.send(msg));
  }
}

module.exports = CleverBotChannel;
