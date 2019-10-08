'use strict';
const Discord = require('discord.js');
const Logger = require('./Logger.js');
const GuildManager = require('./GuildManager.js');
const Command = require('./Command.js');
const path = require('path');
const { mkdir } = require('./util/fswrapper.js');

class Bot {
  constructor(options) {
    options = mergeDefault(Bot.defaultOptions, options);

    this.client = new Discord.Client(options.discord);
    this.config = options.config;

    if (!this.config) throw new Error('You must provide a valid config object');

    this.dataDir = options.data.dir;
    this.logger = new Logger(path.join(this.dataDir, options.data.logger), {
      logToConsole: true,
      logPath: path.join(this.dataDir, 'logs')
    });

    this.ready = false;

    this.guildManagers = new Map();
    this.commands = new Map();
  }

  get guildDataDir() {
    return path.join(this.dataDir, 'guilds');
  }

  async loadGuild(id) {
    const manager = await GuildManager.load(this.guildDataDir, id);
    this.guildManagers.set(id, manager);
  }
}

Bot.defaultOptions = {
  discord: {},
  data: {
    dir: './data',
    logger: 'main.log'
  },
  config: null
};

module.exports = Bot;

function mergeDefault(def, given) {
  if (!given) return def;
  for (const key in def) {
    if (!{}.hasOwnProperty.call(given, key)) {
      given[key] = def[key];
    } else if (given[key] === Object(given[key])) {
      given[key] = mergeDefault(def[key], given[key]);
    }
  }

  return given;
}