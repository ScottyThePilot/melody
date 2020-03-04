'use strict';
const { RichEmbed } = require('discord.js');
const config = require('./config.json');
const { capFirst } = require('./utils/util.js');



class CommandHelp extends RichEmbed {
  constructor(command) {
    super();
    this.setTitle(capFirst(command.name));
    this.setDescription(command.help.long);
    this.setColor([114, 137, 218]);
    this.addField('Usage', blockify(command.help.usage, '\n'));
    this.addField('Example', blockify(command.help.example, '\n'));
    this.addField('Aliases', blockify(command.aliases, ', ', config.prefix), true);
    this.addField('Permissions', command.levelString, true);
  }
}

function blockify(val, sep, p = '') {
  if (typeof val === 'string') val = [val];
  return val.map((v) => '\`' + p + v + '\`').join(sep);
}



class CommandHelpList extends RichEmbed {
  constructor(commands, level, title) {
    super();
    const body = [...commands.values()]
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



module.exports = {
  CommandHelp,
  CommandHelpList
};
