'use strict';
const path = require('path');
const Discord = require('discord.js');
const EventEmitter = require('events');
const Logger = require('./Logger.js');
const GuildManager = require('./GuildManager.js');
const Command = require('./Command.js');
const Collection = require('./Collection.js');
const { exists, mkdir } = require('../utils/fs.js');
const { mergeDefault } = require('../utils/obj.js');
const { awaitEvent, wait } = require('../utils/util.js');

const events = Object.values(Discord.Constants.Events);

class Bot extends EventEmitter {
  /**
   * Creates a new Bot instance
   * @param {BotOptions} [opts] 
   */
  constructor(opts) {
    super();
    const options = mergeDefault(Bot.defaultOptions, opts);

    if (!options.config) throw new Error('No config object provided');

    /** @type {Discord.Client} */
    this.client = new Discord.Client(options.client);

    const { version, token, prefix, owner, trustedUsers = [] } = options.config;
    if (!token) throw new Error('No token provided');
    if (!prefix) throw new Error('No prefix provided');

    for (let p of ['data', 'guilds', 'commands'])
      if (!options.paths[p]) throw new Error(`No ${p} path provided`);

    /** @type {{data: string, guilds: string, commands: string}} */
    this.paths = options.paths;
    
    /** @type {string} */
    this.version = version;
    
    /** @type {string} */
    this.token = token;
    
    /** @type {string} */
    this.prefix = prefix;
    
    /** @type {string} */
    this.owner = owner;

    /** @type {string[]} */
    this.trustedUsers = trustedUsers;

    /** @type {Collection<Command>} */
    this.commands = new Collection();

    /** @type {Map<string, GuildManager>} */
    this.managers = new Map();
  }

  async init() {
    for (let p of ['data', 'guilds', 'commands'])
      if (!await exists(this.paths[p])) await mkdir(this.paths[p]);

    this.logger = new Logger(path.join(this.paths.data, 'main.log'), {
      core: path.join(this.paths.data, 'logs'),
      console: true
    });

    await Promise.all([
      awaitEvent(this.client, 'ready'),
      this.client.login(this.token)
    ]);

    await wait(1000);

    for (let event of events) {
      if (event === 'message') continue;
      this.client.on(event, (...args) => {
        this.emit(event, ...args);
      });
    }

    this.client.on('message', (message) => {
      const parsed = this.parseCommand(message);

      if (parsed) {
        this.emit('command', parsed);
      } else {
        this.emit('message', message);
      }
    });
  }

  get mention() {
    return new RegExp(`^<@!?${this.client.user.id}>\\s*`);
  }

  parseCommand(message, prefixOverride) {
    if (message.author.bot) return null;
  
    const content = message.content.trim();
    const prefix = prefixOverride || this.prefix;
    if (!content.startsWith(prefix)) return null;
  
    // Dissallow whitespace between the prefix and command name
    if (/^\s+/.test(content.slice(prefix.length))) return null;
  
    let args = content.slice(prefix.length).trim().split(/\s+/g);
    const command = args.shift().toLowerCase();
    const argsText = content.slice(prefix.length + command.length).trim();
  
    return { message, command, args, argsText };
  }

  /** @returns {Promise} */
  send(channel, ...args) {
    return channel.send(...args).catch(() => null);
  }

  async loadManager(id) {
    const folder = path.join(this.paths.data, 'guilds');
    const manager = await GuildManager.load(id, folder);
    if (manager.logger.rotation) await manager.logger.checkRotation();
    this.managers.set(id, manager);
  }

  async unloadManager(id) {
    await this.managers.get(id).unload();
  }

  async loadCommandAt(location) {
    const command = requireRoot(location);
    if (command instanceof Command) this.commands.add(command);
  }
}

Bot.defaultOptions = {
  config: null,
  client: {},
  paths: {
    data: null,
    guilds: null,
    commands: null
  }
};

function requireRoot(id) {
  return require(path.join(process.cwd(), id));
}

module.exports = Bot;

/**
 * @typedef BotOptions
 * @property {object} config
 * @property {object} client
 */

/**
 * @typedef {import('./Command.js')} Command
 */