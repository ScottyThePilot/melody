'use strict';
const Command = require('../structures/Command.js');
const config = require('../config.json');

module.exports = new Command({
  name: 'restart',
  level: 3,
  plugin: 'owner',
  help: {
    short: 'Restarts the bot.',
    long: 'Stops the bot, waits 10 seconds, and then restarts the bot.',
    usage: `${config.prefix}restart`,
    example: `${config.prefix}restart`
  },
  run: async function ({ melody, message }) {
    await message.react(String.fromCharCode(0x2705)).catch();

    await melody.destroy();

    // Exit with 0 to signal that the process should restart
    process.exit(0);
  }
});
