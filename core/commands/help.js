'use strict';
const Command = require('../structures/Command.js');
const config = require('../config.json');
const util = require('../modules/util.js');
const { RichEmbed } = require('discord.js');

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
    const { melody, message, trusted } = bundle;
    const msgFailCatcher = util.makeCatcher(melody.logger, 'Unable to send message');
    const args = bundle.args.map((arg) => arg.toLowerCase());

    const isHelpAll = bundle.command === 'helpall';

    const commandListFull = [...melody.commands.values()]
      .sort((a, b) => a.name < b.name ? -1 : a.name > b.name ? 1 : 0);

    const plugins = isHelpAll && !trusted ? await melody.getAccessiblePlugins(message.author)
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
      
      const extra = !isHelpAll ? `Type \`${config.prefix}helpall\` for a list of all commands available to you from other servers.`
        : `Type \`${config.prefix}help\` anywhere for a list of all commands usable there.`;
      
      const from = isHelpAll ? ' (All)'
        : message.guild ? ' (' + message.guild.name + ')'
        : '';

      const embed = new RichEmbed();

      embed.setTitle('Command Help' + from);
      embed.setDescription(`Below is a list of **${what}** and a short description of what they do.\nType \`${config.prefix}help <command>\` for more info about a command.\n${extra}`);
      embed.setColor([114, 137, 218]);
      embed.addField('Command List', body);
      embed.setFooter(`Melody v${config.version[1]} ${config.version[0]}`);

      await message.author.send(embed).catch(msgFailCatcher);
    } else {
      const cmd = commandList.find((command) => {
        return command.name === args[0] || command.aliases.includes(args[0]);
      });

      if (cmd) {
        const embed = new RichEmbed();

        const aliases = cmd.aliases.map((a) => '\`' + config.prefix + a + '\`').join(', ').trim() || 'None';

        embed.setTitle(util.capFirst(cmd.name));
        embed.setDescription(cmd.help.long);
        embed.setColor([114, 137, 218]);
        embed.addField('Usage', '\`' + cmd.help.usage + '\`');
        embed.addField('Example', '\`' + cmd.help.example + '\`', true);
        embed.addField('Aliases', aliases);
        embed.addField('Plugin', '\`' + cmd.plugin.toUpperCase() + '\`');
        embed.addField('Permissions', cmd.help.perms || permissions[cmd.level] || 'Custom');

        await message.channel.send(embed).catch(msgFailCatcher);
      } else {
        await message.channel.send('I can\'t find that command here.');
      }
    }
  }
});
