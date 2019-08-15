'use strict';
const Command = require('../modules/Command.js');
const { msgFailCatcher } = require('../modules/Logger.js');
const config = require('../config.json');

module.exports = new Command({
  name: 'ping',
  help: {
    short: 'Send some feedback.',
    long: 'Sends a message to the bot owner. Feel free to leave any suggestions, questions, comments, or criticism you have.',
    usage: `${config.prefix}feedback [message]`,
    example: `${config.prefix}feedback I like this bot!`
  },
  run: async function (bundle) {
    const { message, client } = bundle;
    
  }
});