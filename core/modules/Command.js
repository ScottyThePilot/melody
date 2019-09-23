'use strict';
const config = require('../config.json');
const Util = require('./util/Util.js');
const Logger = require('./Logger.js');
const { readdir } = require('./util/fswrapper.js');
const controller = require('./controller.js');

const defaultOptions = {
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

class Command {
  constructor(options) {
    let o = Util.mergeDefault(defaultOptions, options);
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
    // Exit silently if the user is blacklisted
    // 0xe1: Ignored: [User is blacklisted]
    if ((await controller.blacklist.get()).includes(bundle.message.author.id)) return 0xe1;

    let isTrusted = [config.ownerID, ...config.trustedUsers].includes(bundle.message.author.id)
    let plugins = bundle.manager ? await bundle.manager.configdb.get('plugins') : Command.pluginsDM;

    // Exit silently if this command's plugin is not enabled in the given server
    // 0xe0: Ignored: [Command not on plugin list]
    if (!plugins.includes(this.plugin) && this.plugin !== 'owner') return 0xe0;

    // Clone bundle and insert userLevel
    let newBundle = Object.assign({
      trusted: isTrusted,
      plugins: plugins,
      userLevel: Command.getUserLevel(bundle)
    }, bundle);

    if (this.disabled) {
      Logger.main.log('USER', `Denying Access to Command ${this.name} for user ${Logger.logifyUser(newBundle.message.author)}`, 'Reason: Command Disabled');
      await Command.sendMessage(newBundle.message.channel, 'That command is disabled.');
      // 0xf0: Rejected [Command Disabled]
      return 0xf0;
    } else if (!this.inGuild && newBundle.message.guild) {
      Logger.main.log('USER', `Denying Access to Command ${this.name} for user ${Logger.logifyUser(newBundle.message.author)}`, 'Reason: Command is Dissallowed in Guilds');
      await Command.sendMessage(newBundle.message.channel, 'You cannot use this command in a Guild, try it in DM.');
      // 0xf1: Rejected [Command is Dissallowed in Guild]
      return 0xf1;
    } else if (!this.inDM && !newBundle.message.guild) {
      Logger.main.log('USER', `Denying Access to Command ${this.name} for user ${Logger.logifyUser(newBundle.message.author)}`, 'Reason: Command is Dissallowed in DM');
      await Command.sendMessage(newBundle.message.channel, 'You cannot use this command in DM, try it in a Guild.');
      // 0xf2: Rejected [Command is Dissallowed in DM]
      return 0xf2;
    } else if (this.level > newBundle.userLevel) {
      Logger.main.log('USER', `Denying Access to Command ${this.name} for user ${Logger.logifyUser(newBundle.message.author)}`, 'Reason: Insufficient Permissions');
      await Command.sendMessage(newBundle.message.channel, 'You do not have permission to do that.');
      // 0xf3: Rejected [Insufficient Permissions]
      return 0xf3;
    } else {
      Logger.main.log('USER', `Running Command ${this.name} for user ${Logger.logifyUser(newBundle.message.author)}`);
      await this.run(newBundle);
      // 0xd0: Accepted
      return 0xd0;
    }
  }

  save() {
    Command.manifest.set(this.name, this);
  }

  static find(alias) {
    for (let [name, command] of Command.manifest) {
      if (name.toLowerCase() === alias.toLowerCase() || command.aliases.includes(alias.toLowerCase())) return command;
    }
    return null;
  }

  static async buildManifest() {
    if (Command.manifest.size > 0) throw new Error('Manifest already built');
    let commandFiles = await readdir('./core/commands');
    commandFiles.forEach((fileName) => {
      if (fileName instanceof Buffer) fileName = fileName.toString();
      require('../commands/' + fileName).save();
    });
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

  static sendMessage(channel, ...args) {
    return channel.send(...args).catch(Logger.msgFailCatcher);
  }
}

Command.pluginsDM = ['general', 'core'];
Command.pluginsAll = ['general', 'core', 'owner'];
Command.manifest = new Map(); // Lists each command once

module.exports = Command;
