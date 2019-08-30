'use strict';
const Command = require('../modules/Command.js');
const config = require('../config.json');

module.exports = new Command({
  name: 'restart',
  level: 10,
  plugin: 'owner',
  help: {
    short: 'Logs the bot off of Discord.',
    long: 'Stops the bot, waits 10 seconds, and then restarts the bot.',
    usage: `${config.prefix}restart`,
    example: `${config.prefix}restart`
  },
  run: async function (bundle) {
    const { message, controller } = bundle;

    await message.react(String.fromCharCode(0x2705)).catch();

    await controller.destroyBot();

    // Exit with 0 to signal that the process should restart
    process.exit(0);
  }
});