'use strict';
import Command from '../core/Command.js';
import config from '../config.js';

export default new Command({
  name: 'stop',
  level: 10,
  help: {
    short: 'Logs the bot off of Discord.',
    long: 'Logs out, terminates the connection to Discord, and destroys the client.',
    usage: `${config.prefix}stop`,
    example: `${config.prefix}stop`
  },
  exec: async function exec({ melody, message }) {
    await message.react('\u2705').catch(() => null);
    await melody.destroy();

    // Exit with 1 to signal that the process should not restart
    process.exit(1);
  }
});
