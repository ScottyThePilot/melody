'use strict';
const { RichEmbed } = require('discord.js');
const config = require('../config.json');
const util = require('../modules/util.js');
const Result = require('./Result.js');

const permissions = {
  [-1]: null,
  [0]: 'Everyone',
  [1]: 'Server administrators',
  [2]: 'Server owners',
  [3]: 'Trusted users',
  [10]: 'Bot owner'
};

class Command {
  constructor(options) {
    let o = util.mergeDefault(Command.defaultOptions, options);
    this.name = o.name;
    this.disabled = o.disabled;
    this.level = o.level;
    this.plugin = o.plugin;
    this.help = o.help;
    this.aliases = o.aliases;
    this.inDM = o.inDM;
    this.inGuild = o.inGuild;
    this.run = o.run;
    this.embed = Command.getHelpEmbed(this);
  }

  async attempt(bundle, logger) {
    let isTrusted = [config.ownerID, ...config.trustedUsers].includes(bundle.message.author.id);
    let plugins = bundle.manager ? Command.getPlugins(bundle.manager.configdb.getSync('plugins')) : Command.pluginsDM;

    // Exit silently if this command's plugin is not enabled in the given server
    if (!plugins.includes(this.plugin) && this.plugin !== 'owner') return new Result.Err('no_command');

    // Clone bundle and insert userLevel
    let newBundle = Object.assign({
      trusted: isTrusted,
      plugins: plugins,
      userLevel: Command.getUserLevel(bundle)
    }, bundle);

    const msgFailCatcher = util.makeCatcher(logger, 'Unable to send message');

    if (this.disabled) {
      logger.log(
        'USER',
        `Denying Access to Command ${this.name} for user ${util.logifyUser(newBundle.message.author)}`,
        'Reason: Command Disabled'
      );

      await newBundle.message.channel.send('That command is disabled.').catch(msgFailCatcher);

      return new Result.Err('disabled');
    } else if (!this.inGuild && newBundle.message.guild) {
      logger.log(
        'USER',
        `Denying Access to Command ${this.name} for user ${util.logifyUser(newBundle.message.author)}`,
        'Reason: Command is Dissallowed in Guilds'
      );

      await newBundle.message.channel.send('You cannot use this command in a Guild, try it in DM.').catch(msgFailCatcher);

      return new Result.Err('no_guild');
    } else if (!this.inDM && !newBundle.message.guild) {
      logger.log(
        'USER',
        `Denying Access to Command ${this.name} for user ${util.logifyUser(newBundle.message.author)}`,
        'Reason: Command is Dissallowed in DM'
      );

      await newBundle.message.channel.send('You cannot use this command in DM, try it in a Guild.').catch(msgFailCatcher);

      return new Result.Err('no_dm');
    } else if (this.level > newBundle.userLevel) {
      logger.log(
        'USER',
        `Denying Access to Command ${this.name} for user ${util.logifyUser(newBundle.message.author)}`,
        'Reason: Insufficient Permissions'
      );

      await newBundle.message.channel.send('You do not have permission to do that.').catch(msgFailCatcher);

      return new Result.Err('no_perm');
    } else {
      logger.log('USER', `Running Command ${this.name} for user ${util.logifyUser(newBundle.message.author)}`);

      await this.run(newBundle);

      return new Result.Ok();
    }
  }

  static getHelpEmbed(command) {
    const embed = new RichEmbed();

    const aliases = command.aliases.map((a) => `\`${config.prefix}${a}\``).join(', ') || 'None';

    embed.setTitle(util.capFirst(command.name));
    embed.setDescription(command.help.long);
    embed.setColor([114, 137, 218]);
    embed.addField('Usage', '\`' + command.help.usage + '\`');
    embed.addField('Example', '\`' + command.help.example + '\`', true);
    embed.addField('Aliases', aliases);
    embed.addField('Plugin', '\`' + command.plugin.toUpperCase() + '\`');
    embed.addField('Permissions', command.help.perms || permissions[command.level] || 'Custom');
  }

  static getUserLevel(bundle) {
    let userLevel = 0;

    if (bundle.message.guild) {
      if (bundle.message.member.hasPermission('ADMINISTRATOR')) {
        userLevel = 1;
      }
  
      if (bundle.message.guild.owner.id === bundle.message.author.id) {
        userLevel = 2;
      }
    } else {
      if (config.trustedUsers.includes(bundle.message.author.id)) {
        userLevel = 3;
      }
    }
  
    if (config.ownerID === bundle.message.author.id) {
      userLevel = 10;
    }

    // 0: Standard User
    // 1: Server Administrator
    // 2: Server Owner
    // 3: Trusted User
    // 10: Bot Owner

    return userLevel;
  }

  static getPlugins(pluginMap) {
    let plugins = [];
    for (let plugin in pluginMap) {
      if (pluginMap[plugin]) plugins.push(plugin);
    }
    return plugins;
  }

  static async inquire(message, question, choices) {
    const block = '0: Close Menu\n' + choices.map((c, i) => `${i + 1}: ${c}`).join('\n');
    const inquiryContents = `${question}\nRespond with the appropriate number:\n\`\`\`\n${block}\n\`\`\``;

    const inquiry = await message.channel.send(inquiryContents);

    const filter = (m) => /^\d+$/.test(m.content.trim());
    const msg = message.channel.awaitMessages(filter, {
      max: 1,
      time: 15000,
      errors: ['time']
    });

    try {
      await msg;
    } catch (err) {
      return [null, null];
    } finally {
      const choice = +(await msg).first().contents;

      if (choice === 0) {
        await inquiry.edit('Menu Closed').catch();
        return [inquiry, null];
      }

      return [inquiry, choice - 1];
    }
  }
}

Command.pluginDefaults = {
  core: true,
  owner: true,
  moderation: true,
  fun: true
};

Command.pluginsAll = ['core', 'owner', 'moderation', 'fun'];
Command.pluginsDM = ['core', 'moderation', 'fun'];

Command.defaultOptions = {
  name: 'default',
  disabled: false,
  level: 0,
  plugin: 'core',
  help: {
    short: '',
    long: '',
    usage: `${config.prefix}`,
    example: `${config.prefix}`
  },
  aliases: [],
  inDM: true,
  inGuild: true,
  run: function () {
    throw new Error('Run was not supplied!');
  }
};

module.exports = Command;
