'use strict';
const path = require('path');
const { mkdir, exists } = require('../modules/fswrapper.js');
const Datastore = require('./Datastore.js');
const Command = require('./Command.js');
const Logger = require('./Logger.js');

class GuildManager {
  // Loads a new GuildManager
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

    // Try to fill in empty settings to replace missing config properties
    await this.repairConfigDB(configdb);
    
    return new GuildManager(id, logger, configdb);
  }

  // Creates the assigned directory if it doesn't exist
  static async mount(location, id) {
    if (!GuildManager.exists(location, id)) await mkdir(path.join(location, id));
  }

  // Unloads the GuildManager so it can be safely removed
  async unload() {
    await this.logger.end();
  }

  // Checks whether a guild is stored or not
  static exists(location, id) {
    return exists(path.join(location, id));
  }

  constructor(id, logger, configdb) {
    this.id = id;
    this.logger = logger;
    this.configdb = configdb;
    //this.autoModContext = new Map();
  }

  static isIncompleteState(state) {
    for (let prop in this.defaultConfig)
      if (!state.hasOwnProperty(prop)) return true;
    return false;
  }

  static async repairConfigDB(configdb) {
    if (!this.isIncompleteState(await configdb.get())) return;
    await configdb.edit((state) => {
      console.log('Repairing State');
      for (let prop in this.defaultConfig)
        if (!state.hasOwnProperty(prop))
          state[prop] = this.defaultConfig[prop];
    });
  }

  log(...args) {
    return this.logger.log(...args);
  }
}

GuildManager.defaultConfig = {
  plugins: Command.pluginDefaults,
  logMessages: false,
  logMessageChanges: false,
  mutedRole: null,
  cleverBotZones: []
};

module.exports = GuildManager;
