'use strict';
const Command = require('../core/structures/Command.js');
const Embeds = require('../Embeds.js');
const config = require('../config.js');

module.exports = new Command({
  name: 'help',
  help: {
    short: 'Gets command help.',
    long: 'Gets help for a command or all of Melody\'s commands.',
    usage: `${config.prefix}help [command]`,
    example: `${config.prefix}help ping`
  },
  aliases: ['helpall', 'halp', 'h'],
  run: async function run({ melody, message, level, args, command }) {
    for (const [arg, i] of args.entries()) args[i] = arg.toLowerCase();

    if (!args[0] || command === 'helpall') {
      const embed = new Embeds.CommandHelpList(melody.commands, level);
      await message.author.send(embed).catch(melody.catcher);
    } else {
      const cmd = Command.find(melody.commands, args[0]);
      if (!cmd) {
        await message.author.send('I can\'t find that command.').catch(melody.catcher);
      } else {
        const embed = new Embeds.CommandHelp(cmd);
        await message.author.send(embed).catch(melody.catcher);
      }
    }
  }
});
