'use strict';
import Command from '../core/Command.js';
import * as Embeds from '../core/Embeds.js';
import config from '../config.js';

export default new Command({
  name: 'clever',
  help: {
    short: 'Manages cleverbot.',
    long: 'Allows management of the cleverbot feature.',
    usage: `${config.prefix}clever <'dump'|'clear'>`,
    example: `${config.prefix}ping`
  },
  level: 10,
  hidden: true,
  exec: async function exec({ melody, message, args }) {
    if (!args[0]) {
      await message.channel.send('No subcommand provided. Try \`dump\` or \`clear\`.').catch(melody.catcher);
    } else if (args[0] === 'dump') {
      const channel = melody.clever.get(message.channel.id);
      if (channel || channel.history.length === 0) {
        await message.channel.send(`No CleverBot context for channel \`${id}\`.`).catch(melody.catcher);
      } else {
        const list = new Embeds.List(channel.history);
        await message.channel.send(`CleverBot context for channel \`${id}\`:`, list).catch(melody.catcher);
      }
    } else if (args[0] === 'clear') {
      const id = message.channel.id;
      await melody.clever.clear(id);
      await message.channel.send(`Cleared CleverBot context for channel \`${id}\`.`).catch(melody.catcher);
    } else {
      await message.channel.send('Invalid subcommand. Try \`dump\` or \`clear\`.').catch(melody.catcher);
    }
  }
});
