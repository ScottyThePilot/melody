'use strict';
const Command = require('../modules/Command.js');
const config = require('../config.json');
const util = require('../modules/util/util.js');

const rxID = /[0-9]+/;
const rxTag = /@?(.+#[0-9]{4})/;

module.exports = new Command({
  name: 'ban',
  plugin: 'moderation',
  help: {
    short: 'Bans a user.',
    long: 'Bans a user with the given reason.',
    usage: `${config.prefix}ban <user> [reason]`,
    example: `${config.prefix}ban @User#0000 They were not being nice.`,
    perms: 'Anyone with \`BAN_MEMBERS\` permission'
  },
  inDM: false,
  run: async function ({ melody, message }) {
    const msgFailCatcher = util.makeCatcher(melody.logger, 'Unable to send message');
  }
});
