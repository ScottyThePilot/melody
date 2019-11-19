'use strict';
const Command = require('../structures/Command.js');
const config = require('../config.json');
const util = require('../modules/util.js');

module.exports = new Command({
  name: 'mute',
  level: 0,
  plugin: 'moderation',
  help: {
    short: 'Mutes a user.',
    long: `Gives a user the current server\'s muted role. Make sure to set up your server\'s muted role with \`${config.prefix}configure set mutedRole <role id>\`.`,
    usage: `${config.prefix}mute <@mention|user id> [duration]`,
    example: `${config.prefix}mute @User#0000 1 day`
  },
  inDM: false,
  aliases: ['unmute'],
  run: async function ({ melody, message, args, command }) {
    const msgFailCatcher = util.makeCatcher(melody.logger, 'Unable to send message');

    if (command === 'mute') {

    } else {
      
    }
  }
});
