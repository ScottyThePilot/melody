'use strict';
const Command = require('../core/Command.js');
const config = require('../config.json');

module.exports = new Command({
  name: 'ping',
  help: {
    short: 'Gets the current ping.',
    long: 'Gets the current latency.',
    usage: `${config.prefix}ping`,
    example: `${config.prefix}ping`
  },
  run: async function run({ melody, message }) {
    const msg = await message.channel.send('Ping?').catch(melody.catcher);
    const lat1 = (msg.createdTimestamp - message.createdTimestamp).toFixed(2);
    const lat2 = melody.client.ping.toFixed(2);
    await msg.edit(`Pong! Latency is \`${lat1}ms\`. API Latency is \`${lat2}ms\``).catch(melody.catcher);
  }
});
