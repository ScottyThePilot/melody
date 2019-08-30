'use strict';
const Command = require('../modules/Command.js');
const config = require('../config.json');

module.exports = new Command({
  name: 'stop',
  level: 10,
  plugin: 'owner',
  help: {
    short: 'Logs the bot off of Discord.',
    long: 'Logs out, terminates the connection to Discord, and destroys the client.',
    usage: `${config.prefix}stop`,
    example: `${config.prefix}stop`
  },
  run: async function (bundle) {
    const { message, controller } = bundle;

    await message.react(String.fromCharCode(0x2705)).catch();

    await controller.destroyBot();

    // Exit with 0 to signal that the process should not restart
    process.exit(1);
  }
});
