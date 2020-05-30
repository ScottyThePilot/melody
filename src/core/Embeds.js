'use strict';
import Util from '../utils/Util.js';
import config from '../config.js';
import Discord from 'discord.js';

export class CommandHelp extends Discord.MessageEmbed {
  constructor(command) {
    super();
    this.setTitle(Util.capFirst(command.name));
    this.setDescription(command.help.long);
    this.setColor([114, 137, 218]);
    this.addField('Usage', CommandHelp.blockify(command.help.usage, '\n'), true);
    this.addField('Example', CommandHelp.blockify(command.help.example, '\n'), true);
    this.addField('Aliases', CommandHelp.blockify(command.aliases, ', ', config.prefix) || 'none', true);
    this.addField('Permissions', command.group, true);
  }

  static blockify(val, sep, p = '') {
    if (!Array.isArray(val)) val = [val];
    return val.map((v) => '\`' + p + v + '\`').join(sep);
  }
}

export class CommandHelpList extends Discord.MessageEmbed {
  constructor(commands, level, title) {
    super();
    const body = commands.array()
      .filter((command) => command.level <= level)
      .sort((a, b) => a.name < b.name ? -1 : a.name > b.name ? 1 : 0)
      .map((command) => `\`${config.prefix}${command.name}\`: *${command.help.short}*`)
      .join('\n');
    this.setTitle('Command Help');
    this.setDescription(
      'Below is a list of commands, each with short description of what they do.\n' +
      `Type \`${config.prefix}help <command>\` for more info about a command.\n` +
      'This list only shows commands you have access to.'
    );
    this.setColor([114, 137, 218]);
    this.addField('Command List' + (title ? ' (' + title + ')' : ''), body);
    this.setFooter(`Melody v${config.version}`);
  }
}
