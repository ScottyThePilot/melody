'use strict';
const config = require('../config.js');
const Util = require('./util/Util.js');
const Logger = require('./Logger.js');

const defaultOptions = {
  name: 'Default',
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

const pluginsDM = ['general', 'core'];

class Command {
  static create(options) {
    const saved = new this(options);
    Command.manifest.set(saved.name, saved);
    return saved;
  }

  constructor(options) {
    var o = Util.mergeDefault(defaultOptions, options);
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

  async attempt(bundle) {
    var plugins = bundle.manager ? await bundle.manager.configdb.get('plugins') : pluginsDM;

    // Exit silently if this command's plugin is not enabled in the given server
    // 0xe0: Ignored: [Command not on plugin list]
    if (!plugins.includes(o.plugin)) return 0xe0;

    // Clone bundle and insert userLevel
    var newBundle = Object.assign({ userLevel: Command.getUserLevel(bundle) }, bundle);

    if (o.disabled) {
      Logger.main.log('USER', `Denying Access to Command ${command} for user ${Util.logifyUser(message.author)}`, 'Reason: Command Disabled');
      await Command.sendMessage(newBundle.message.channel, 'That command is disabled.');
      // 0xf0: Rejected [Command Disabled]
      return 0xf0;
    } else if (!this.inGuild && newBundle.message.guild) {
      Logger.main.log('USER', `Denying Access to Command ${command} for user ${Util.logifyUser(message.author)}`, 'Reason: Command is Dissallowed in Guilds');
      await Command.sendMessage(newBundle.message.channel, 'You cannot use this command in a Guild, try it in DM.');
      // 0xf1: Rejected [Command is Dissallowed in Guild]
      return 0xf1;
    } else if (!this.inDM && !newBundle.message.guild) {
      Logger.main.log('USER', `Denying Access to Command ${command} for user ${Util.logifyUser(message.author)}`, 'Reason: Command is Dissallowed in DM');
      await Command.sendMessage(newBundle.message.channel, 'You cannot use this command in DM, try it in a Guild.');
      // 0xf2: Rejected [Command is Dissallowed in DM]
      return 0xf2;
    } else if (this.level > newBundle.userLevel) {
      Logger.main.log('USER', `Denying Access to Command ${command} for user ${Util.logifyUser(message.author)}`, 'Reason: Insufficient Permissions');
      await Command.sendMessage(newBundle.message.channel, 'You do not have permission to do that.');
      // 0xf3: Rejected [Insufficient Permissions]
    } else {
      Logger.main.log('USER', `Running Command ${command} for user ${Util.logifyUser(message.author)}`);
      await this.run(newBundle);
      // 0xd0: Accepted
      return 0xd0;
    }
  }

  static find(alias) {
    for (var [name, command] of Command.manifest) {
      if (name === alias && command.aliases.includes(alias)) return command;
    }
    return null;
  }

  static getUserLevel(bundle) {
    var userLevel = 0;

    if (bundle.message.guild) {
      if (bundle.message.member.hasPermission('ADMINISTRATOR')) {
        userLevel = 1;
      }
  
      if (bundle.message.guild.owner.id === bundle.message.author.id) {
        userLevel = 2;
      }
    } else {
      if (bundle.config.trustedIDs.includes(bundle.message.author.id)) {
        userLevel = 3;
      }
    }
  
    if (bundle.config.ownerID === bundle.message.author.id) {
      userLevel = 10;
    }

    return userLevel;
  }

  static sendMessage(channel, ...args) {
    return channel.send(...args).catch(Logger.msgFailCatcher);
  }
}

Command.manifest = new Map(); // Lists each command once

module.exports = Command;