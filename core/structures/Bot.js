'use strict';
const Discord = require('discord.js');
const Logger = require('./Logger.js');
const GuildManager = require('./GuildManager.js');
const Command = require('./Command.js');
const { exists, mkdir, readdir } = require('../modules/fswrapper.js');
const util = require('../modules/util.js');
const path = require('path');

class Bot {
  constructor(options) {
    options = util.mergeDefault(Bot.defaultOptions, options);

    this.client = new Discord.Client(options.discord);
    this.config = options.config;

    if (!this.config) throw new Error('You must provide a valid config object');

    this.paths = {
      data: options.paths.data,
      commands: options.paths.commands
    };

    this.logger = null;

    this.ready = false;

    this.guildManagers = new Map();
    this.commands = new Map();
  }

  get mention() {
    if (!this.ready) return null;
    return new RegExp(`^<@!?${this.client.user.id}>\\s*`);
  }

  async init() {
    const noExist = !exists(this.paths.data);
    if (noExist) await mkdir(this.paths.data);

    this.logger = new Logger(path.join(this.paths.data, 'main.log'), {
      logToConsole: true,
      logPath: path.join(this.paths.data, 'logs')
    });

    const guildsPath = path.join(this.paths.data, 'guilds');
    if (noExist || !exists(guildsPath)) await mkdir(guildsPath);

    await this.logger.checkRotation();
    this.logger.log('Begin Log');
  }

  async loadGuild(id) {
    const manager = await GuildManager.load(path.join(this.paths.data, 'guilds'), id);
    if (manager.logger.rotation) manager.logger.checkRotation(this.logger);
    this.guildManagers.set(id, manager);
  }

  async unloadGuild(id) {
    await this.guildManagers.get(id).unload();
  }

  async buildCommands() {
    if (this.commands.size > 0) throw new Error('Commands already built');
    (await readdir(this.paths.commands)).forEach((fileName) => {
      const command = require(path.join('../../', this.paths.commands, fileName.toString()));
      if (command instanceof Command) this.commands.set(command.name, command);
    });
  }

  async destroy() {
    this.log('INFO', 'Shutting Down...');


    for (let guild of this.client.guilds.values()) {
      await this.unloadGuild(guild.id);
      this.log('DATA', `Guild ${util.logifyGuild(guild)} unloaded`);
    }
  
    await this.logger.end();
    
    await this.client.destroy();
  }

  findCommand(alias) {
    for (let [name, command] of this.commands) {
      if (name.toLowerCase() === alias.toLowerCase() ||
        command.aliases.includes(alias.toLowerCase()))
        return command;
    }
    return null;
  }

  getAccessiblePlugins(user) {
    let userPlugins = Command.pluginsDM.slice(0);
    
    for (let manager of this.guildManagers.values()) {
      const guild = this.client.guilds.get(manager.id);
  
      if (!guild.members.has(user.id)) return;
  
      const plugins = manager.configdb.getSync('plugins');
  
      plugins.forEach((plugin) => {
        if (!userPlugins.includes(plugin)) userPlugins.push(plugin);
      });
    }
  
    return userPlugins;
  }

  login() {
    return this.client.login(this.config.token);
  }

  log(...args) {
    return this.logger.log(...args);
  }

  on(eventName, listener) {
    const that = this;
    return this.client.on(eventName, function (...args) {
      if (that.ready) listener.call(this, ...args);
    });
  }
}

Bot.defaultOptions = {
  discord: {},
  paths: {
    data: './data',
    commands: './commands'
  },
  config: null
};

module.exports = Bot;