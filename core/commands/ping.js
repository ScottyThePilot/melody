'use strict';
const Command = require('../modules/Command.js');
const { msgFailCatcher } = require('../modules/Logger.js');
const config = require('../config.json');

module.exports = new Command({
  name: 'ping',
  plugin: 'core',
  help: {
    short: 'Gets the current ping.',
    long: 'Gets the current latency.',
    usage: `${config.prefix}ping`,
    example: `${config.prefix}ping`
  },
  run: async function (bundle) {
    const { message, client } = bundle;
    const msg = await message.channel.send('Ping?').catch(msgFailCatcher);
    await msg.edit(`Pong! Latency is \`${msg.createdTimestamp - message.createdTimestamp}ms\`. API Latency is \`${client.ping.toFixed(2)}ms\``).catch(msgFailCatcher);
  }
});
