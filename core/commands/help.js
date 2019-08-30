'use strict';
const { RichEmbed } = require('discord.js');
const Command = require('../modules/Command.js');
const { msgFailCatcher } = require('../modules/Logger.js');
const config = require('../config.json');
const Util = require('../modules/util/Util.js');

const permissions = {
  [-1]: null,
  [0]: 'Everyone',
  [1]: 'Server administrators',
  [2]: 'Server owners',
  [3]: 'Trusted users',
  [10]: 'Bot owner'
};

module.exports = new Command({
  name: 'help',
  plugin: 'core',
  help: {
    short: 'Gets command help.',
    long: 'Gets help for a command or all the commands.',
    usage: `${config.prefix}help [command]`,
    example: `${config.prefix}help ping`
  },
  aliases: ['helpall', 'halp', 'h'],
  run: async function run(bundle) {
    const { client, message, trusted, controller } = bundle;
    const args = bundle.args.map((arg) => arg.toLowerCase());

    const isHelpAll = bundle.command === 'helpall';

    const commandListFull = [...Command.manifest.values()]
      .sort((a, b) => a.name < b.name ? -1 : a.name > b.name ? 1 : 0);

    const plugins = isHelpAll && !trusted ? await controller.getAccessiblePlugins(message.author, client)
      : bundle.plugins;

    const commandList = (isHelpAll || !message.guild) && trusted ? commandListFull
      : commandListFull.filter((cmd) => plugins.includes(cmd.plugin));
    
    if (!args[0] || isHelpAll) {
      const body = commandList.map((command) => {
        return `\`${config.prefix}${command.name}\`: *${command.help.short}*`;
      }).join('\n');

      const what = isHelpAll && trusted ? 'all my commands'
        : isHelpAll ? 'all commands available to you'
        : message.guild ? 'commands available in ' + message.guild.name
        : 'standard commands';
      
      const extra = !isHelpAll ? 'Type \`;helpall\` for a list of all commands available to you from other servers.'
        : 'Type \`;help\` anywhere for a list of all commands usable there.';
      
      const from = isHelpAll ? ' (All)'
        : message.guild ? ' (' + message.guild.name + ')'
        : '';

      const embed = new RichEmbed();

      embed.setTitle('Command Help' + from);
      embed.setDescription(`Below is a list of ${what} and a short description of what they do.\nType \`;help <command>\` for more info about a command.\n${extra}`);
      embed.setColor([114, 137, 218]);
      embed.addField('Command List', body);

      await message.author.send(embed).catch(msgFailCatcher);
    } else {
      const cmd = commandList.find((command) => {
        return command.name === args[0] || command.aliases.includes(args[0]);
      });

      if (cmd) {
        const embed = new RichEmbed();

        const aliases = cmd.aliases.map((a) => '\`;' + a + '\`').join(', ').trim() || 'None';

        embed.setTitle(Util.capFirst(cmd.name));
        embed.setDescription(cmd.help.long);
        embed.setColor([114, 137, 218]);
        embed.addField('Usage', '\`' + cmd.help.usage + '\`');
        embed.addField('Example', '\`' + cmd.help.example + '\`', true);
        embed.addField('Aliases', aliases);
        embed.addField('Plugin', '\`' + cmd.plugin.toUpperCase() + '\`');
        embed.addField('Permissions', permissions[cmd.level] || this.permissions || 'Custom');

        await message.channel.send(embed).catch(msgFailCatcher);
      } else {
        await message.channel.send('I can\'t find that command here.');
      }
    }
  }
});
