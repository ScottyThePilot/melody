'use strict';
const fs = require('fs');
const Datastore = require('./Datastore.js');
const Logger = require('./Logger.js');
const NodeUtil = require('util');

const mkdir = NodeUtil.promisify(fs.mkdir);
const write = NodeUtil.promisify(fs.writeFile);

class GuildManager {
  static async load(id) {
    await this.mount(id);
    const logger = new Logger(`./data/${id}/latest.log`, {
      logPath: `./data/${id}/logs`
    });
    const configdb = new Datastore(`./data/${id}/guildConfig.json`, {
      data: this.defaultConfig
    });
    const saved = new this(id, logger, configdb);
    GuildManager.all.set(id, saved);
    
    return saved;
  }
  
  static async mount(id) {
    var exists = GuildManager.exists(id);
    if (!exists) {
      await mkdir(`./data/${id}`);
      await mkdir(`./data/${id}/logs`);
      await write(`./data/${id}/latest.log`, '');
      return;
    }
    if (!fs.existsSync(`./data/${id}/latest.log`)) {
      await write(`./data/${id}/latest.log`, '');
    }
  } // Creates the assigned directories and files

  static async unload(id) {
    await GuildManager.all.get(id).logger.end();
    this.all.delete(id);
  }

  static exists(id) {
    return fs.existsSync('./data/' + id);
  } // Checks whether a guild is stored or not

  constructor(id, logger, configdb) {
    this.id = id;
    this.logger = logger;
    this.configdb = configdb;
    this.memory = new Map();
  }

  log(...args) {
    return this.logger.log(...args);
  }
}

GuildManager.all = new Map();

GuildManager.defaultConfig = {
  plugins: ['core', 'general'],
  trackInvites: false,
  preserveRoles: false,
  logMessages: false,
  autoMod: false,
  antiSpam: false,
  mutedRole: null
};

module.exports = GuildManager;