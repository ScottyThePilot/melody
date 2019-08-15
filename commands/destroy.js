'use strict';
const Command = require('../modules/Command.js');
const config = require('../config.json');

module.exports = new Command({
  name: 'destroy',
  level: 10,
  plugin: 'core',
  help: {
    short: 'Logs the bot off of Discord.',
    long: 'Logs out, terminates the connection to Discord, and destroys the client.',
    usage: `${config.prefix}destroy`,
    example: `${config.prefix}destroy`
  },
  run: async function (bundle) {
    const { message, controller } = bundle;

    message.react(String.fromCharCode(0x2705)).catch();

    await controller.destroyBot();
  }
});