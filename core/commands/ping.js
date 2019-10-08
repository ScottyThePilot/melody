'use strict';
const Command = require('../modules/Command.js');
const config = require('../config.json');
const util = require('../modules/util/util.js');

module.exports = new Command({
  name: 'ping',
  plugin: 'core',
  help: {
    short: 'Gets the current ping.',
    long: 'Gets the current latency.',
    usage: `${config.prefix}ping`,
    example: `${config.prefix}ping`
  },
  run: async function ({ melody, message, client }) {
    const msgFailCatcher = util.makeCatcher(melody.logger, 'Unable to send message');

    const msg = await message.channel.send('Ping?').catch(msgFailCatcher);
    const l = msg.createdTimestamp - message.createdTimestamp;
    await msg.edit(`Pong! Latency is \`${l}ms\`. API Latency is \`${client.ping.toFixed(2)}ms\``).catch(msgFailCatcher);
  }
});
