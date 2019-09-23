'use strict';
const Datastore = require('./Datastore.js');
const Logger = require('./Logger.js');
const { mkdir, write, exists } = require('./util/fswrapper.js');

class GuildManager {
  static async load(id) {
    await this.mount(id);
    const logger = new Logger(`./core/data/${id}/latest.log`, {
      logPath: `./core/data/${id}/logs`
    });
    const configdb = new Datastore(`./core/data/${id}/guildConfig.json`, {
      data: this.defaultConfig
    });
    const saved = new this(id, logger, configdb);
    GuildManager.all.set(id, saved);
    
    return saved;
  }
  
  static async mount(id) {
    if (!GuildManager.exists(id)) {
      await mkdir(`./core/data/${id}`);
      await mkdir(`./core/data/${id}/logs`);
      await write(`./core/data/${id}/latest.log`, '');
      return;
    }
    if (!exists(`./core/data/${id}/latest.log`)) {
      await write(`./core/data/${id}/latest.log`, '');
    }
  } // Creates the assigned directories and files

  static async unload(id) {
    await GuildManager.all.get(id).logger.end();
    this.all.delete(id);
  }

  static exists(id) {
    return exists('./core/data/' + id);
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
  logMessageChanges: false,
  autoMod: false,
  antiSpam: false,
  mutedRole: null
};

module.exports = GuildManager;
