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
  inDM: true,
  inGuild: true,
  run: function () {
    throw new Error('Run was not supplied!');
  }
};

class Command {
  static create(options) {
    const saved = new this(options);
    Command.manifest.set(name, saved);
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
    this.run = o.run;
  }

  async attempt(bundle) {
    var plugins = await bundle.manager.configdb.get('plugins');
    if (plugins.includes(o.plugin)) return;

    if (o.disabled) {
      Logger.main.log('USER', `Denying Access to Command ${command} for user ${Util.logifyUser(message.author)}`, 'Reason: Command Disabled');
      
    } else {

    }
  }

  async static buildManifest() {

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
}

Command.manifest = new Map(); // Lists each command once

module.exports = Command;