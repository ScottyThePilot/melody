'use strict';
import Command from '../core/Command.js';
import config from '../config.js';

export default new Command({
  name: 'restart',
  level: 3,
  help: {
    short: 'Restarts the bot.',
    long: 'Stops the bot, waits 10 seconds, and then restarts the bot.',
    usage: `${config.prefix}restart`,
    example: `${config.prefix}restart`
  },
  exec: async function exec({ melody, message }) {
    await message.react('\u2705').catch(() => null);
    await melody.destroy();

    // Exit with 0 to signal that the process should restart
    process.exit(0);
  }
});
