'use strict';
import Command from '../core/Command.js';
import * as Embeds from '../core/Embeds.js';
import config from '../config.js';

export default new Command({
  name: 'help',
  help: {
    short: 'Gets command help.',
    long: 'Gets help for a command or all of Melody\'s commands.',
    usage: `${config.prefix}help [command]`,
    example: `${config.prefix}help ping`
  },
  aliases: ['helplist', 'helpall', 'hlep', 'halp', 'h'],
  exec: async function exec({ melody, message, level, where, args, command }) {
    const all = command === 'helplist' || command === 'helpall';

    if (!args[0] || all) {
      const guild = where === 'guild' ? message.guild.name : null;
      const embed = new Embeds.CommandHelpList(melody.commands, level, guild);
      await message.author.send(embed).catch(melody.catcher);
    } else {
      const cmd = melody.commands.find((c) => c.is(args[0]));
      if (!cmd) {
        await message.channel.send('That command does not exist.').catch(melody.catcher);
      } else {
        const embed = new Embeds.CommandHelp(cmd);
        await message.channel.send(embed).catch(melody.catcher);
      }
    }
  }
});
