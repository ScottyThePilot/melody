'use strict';
const Command = require('../structures/Command.js');
const config = require('../config.json');
const util = require('../modules/util.js');

const aliases = [
  'cleverbotzones',
  'cleverzone',
  'cleverzones',
  'cbzone',
  'cbzones'
];

module.exports = new Command({
  name: 'cleverbotzone',
  plugin: 'fun',
  level: 1,
  disabled: true,
  help: {
    short: 'Turn a channel into a CleverBot zone.',
    long: 'Turns a channel into a CleverBot zone. Melody will send a CleverBot response to all messages sent in CleverBot zones.',
    usage: `${config.prefix}`,
    example: `${config.prefix}`
  },
  aliases,
  inDM: false,
  run: async function ({ melody, message }) {
    const msgFailCatcher = util.makeCatcher(melody.logger, 'Unable to send message');

    // Do stuff in here
  }
});
