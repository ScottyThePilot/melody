'use strict';
const config = require('../config.json');
const util = require('./util/util.js');


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
  }

  async attempt(bundle, logger) {
    let isTrusted = [config.ownerID, ...config.trustedUsers].includes(bundle.message.author.id);
    let plugins = bundle.manager ? bundle.manager.configdb.getSync('plugins') : Command.pluginsDM;

    // Exit silently if this command's plugin is not enabled in the given server
    // 0xe0: Ignored: [Command not on plugin list]
    if (!plugins.includes(this.plugin) && this.plugin !== 'owner') return 0xe0;

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

      // 0xf0: Rejected [Command Disabled]
      return 0xf0;
    } else if (!this.inGuild && newBundle.message.guild) {
      logger.log(
        'USER',
        `Denying Access to Command ${this.name} for user ${util.logifyUser(newBundle.message.author)}`,
        'Reason: Command is Dissallowed in Guilds'
      );

      await newBundle.message.channel.send('You cannot use this command in a Guild, try it in DM.').catch(msgFailCatcher);

      // 0xf1: Rejected [Command is Dissallowed in Guild]
      return 0xf1;
    } else if (!this.inDM && !newBundle.message.guild) {
      logger.log(
        'USER',
        `Denying Access to Command ${this.name} for user ${util.logifyUser(newBundle.message.author)}`,
        'Reason: Command is Dissallowed in DM'
      );

      await newBundle.message.channel.send('You cannot use this command in DM, try it in a Guild.').catch(msgFailCatcher);

      // 0xf2: Rejected [Command is Dissallowed in DM]
      return 0xf2;
    } else if (this.level > newBundle.userLevel) {
      logger.log(
        'USER',
        `Denying Access to Command ${this.name} for user ${util.logifyUser(newBundle.message.author)}`,
        'Reason: Insufficient Permissions'
      );

      await newBundle.message.channel.send('You do not have permission to do that.').catch(msgFailCatcher);

      // 0xf3: Rejected [Insufficient Permissions]
      return 0xf3;
    } else {
      logger.log('USER', `Running Command ${this.name} for user ${util.logifyUser(newBundle.message.author)}`);

      await this.run(newBundle);

      // 0xd0: Accepted
      return 0xd0;
    }
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
}

Command.pluginsDM = ['general', 'core'];
Command.pluginsAll = ['general', 'core', 'owner'];
Command.defaultOptions = {
  name: 'default',
  disabled: false,
  level: 0,
  plugin: 'general',
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
