'use strict';
const Command = require('../structures/Command.js');
const config = require('../config.json');
const util = require('../modules/util.js');

module.exports = new Command({
  //name: 'name_this_command',
  plugin: 'core',
  //disabled: true,
  //level: 0,
  help: {
    short: 'Write a short help description here.',
    long: 'Write a more verbose command description here.',
    usage: `${config.prefix}`,
    example: `${config.prefix}`
  },
  //aliases: [],
  run: async function ({ melody, message }) {
    const msgFailCatcher = util.makeCatcher(melody.logger, 'Unable to send message');

    // Do stuff in here
  }
});
