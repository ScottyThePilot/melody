'use strict';
const path = require('path');
const { mkdir, exists } = require('../modules/fswrapper.js');
const Datastore = require('./Datastore.js');
const Logger = require('./Logger.js');

class GuildManager {
  static async load(location, id) {
    await this.mount(location, id);

    // Logger will create the log file and/or folder if either don't exist
    const logger = new Logger(path.join(location, id, 'latest.log'), {
      logPath: path.join(location, id, 'logs')
    });

    // Datastore will create the db file if it doesn't exist
    const configdb = new Datastore(path.join(location, id, 'guildConfig.json'), {
      defaultData: this.defaultConfig,
      persistence: true
    });
    
    return new GuildManager(id, logger, configdb);
  } // Loads a new GuildManager
  
  static async mount(location, id) {
    if (!GuildManager.exists(location, id)) await mkdir(path.join(location, id));
  } // Creates the assigned directory if it doesn't exist

  async unload() {
    await this.logger.end();
  } // Unloads the GuildManager so it can be safely removed

  static exists(location, id) {
    return exists(path.join(location, id));
  } // Checks whether a guild is stored or not

  constructor(id, logger, configdb) {
    this.id = id;
    this.logger = logger;
    this.configdb = configdb;
    this.autoModContext = new Map();
  }

  log(...args) {
    return this.logger.log(...args);
  }
}

GuildManager.defaultConfig = {
  plugins: ['core', 'general'],
  logMessages: false,
  logMessageChanges: false
};

module.exports = GuildManager;
