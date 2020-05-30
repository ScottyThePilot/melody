'use strict';
import Command from '../core/Command.js';
import config from '../config.js';

export default new Command({
  name: 'ping',
  help: {
    short: 'Gets the current ping.',
    long: 'Gets the current latency.',
    usage: `${config.prefix}ping`,
    example: `${config.prefix}ping`
  },
  exec: async function exec({ melody, message }) {
    const msg = await message.channel.send('Ping?').catch(melody.catcher);
    const l = msg.createdTimestamp - message.createdTimestamp;
    await msg.edit(`Pong! Latency is \`${l}ms\`. API Latency is \`${melody.client.ws.ping.toFixed(2)}ms\``).catch(melody.catcher);
  }
});
