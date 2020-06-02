'use strict';
import Manager from './Manager.js';
import Logger from '../fs/Logger.js';
import CleverBotManager from '../feature/CleverBotManager.js';
import Group from '../utils/Group.js';
import Table from '../utils/Table.js';
import Util from '../utils/Util.js';
import Discord from 'discord.js';
import EventEmitter from 'events';
import path from 'path';
import fs from 'fs';

const events = Object.values(Discord.Constants.Events);

export default class Melody extends EventEmitter {
  /**
   * @param {Discord.Client} client 
   * @param {Options} options 
   */
  constructor(client, options) {
    super();
    const {
      logger,
      commands,
      managers,
      clever,
      config: {
        version,
        token,
        prefix,
        owner,
        trustedUsers,
        dataDir
      }
    } = options;

    /** @type {boolean} */
    this.ready = false;

    /** @type {Discord.Client} */
    this.client = client;
    /** @type {Logger} */
    this.logger = logger;
    /** @type {CleverBotManager} */
    this.clever = clever;
    /** @type {Group<import('./Command').default>} */
    this.commands = commands;
    /** @type {Table<string, Manager>} */
    this.managers = managers;

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
    /** @type {string} */
    this.dataDir = dataDir;
  }

  /**
   * @param {Config} config
   * @param {Group<Command>} [commands]
   */
  static async create(config, commandArray = []) {
    const client = new Discord.Client(config.client);

    await Util.suppressCode(fs.promises.mkdir(config.dataDir, { recursive: true }), 'EEXIST');
    await Util.suppressCode(fs.promises.mkdir(path.join(config.dataDir, 'guilds')), 'EEXIST');
    const logger = await Logger.create(path.join(config.dataDir, 'main.log'), {
      logsFolder: path.join(config.dataDir, 'logs'),
      logToConsole: true
    });

    const clever = new CleverBotManager();

    const managers = new Table();
    const commands = new Group(commandArray);
    return await new Melody(client, { logger, clever, managers, commands, config }).init();
  }

  /**
   * @private
   */
  async init() {
    // Wait for the connection to be established
    await Promise.all([
      Util.onceEvent(this.client, 'ready'),
      this.client.login(this.token)
    ]);

    this.log('INFO', 'Connection established');

    // Wait a bit
    await Util.wait(1000);

    // Register events
    for (const event of events) {
      if (event === 'message') continue;
      this.client.on(event, (...args) => {
        this.emit(event, ...args);
      });
    }

    this.client.on('message', message => {
      const parsed = this.parseCommand(message);
      if (parsed)
        this.emit('command', parsed);
      else
        this.emit('message', message);
    });

    // Register guild managers
    for (const guild of this.client.guilds.cache.values()) {
      await this.loadManager(guild.id);
      this.logger.log('DATA', `Guild ${Util.logifyGuild(guild)} loaded`);
    }

    this.logger.log('DATA', `${this.commands.size} Commands loaded`);
    this.logger.log('INFO', 'Bot ready!');

    this.ready = true;

    return this;
  }

  /** @type {RegExp} */
  get mention() {
    return new RegExp(`^<@!?${this.client.user.id}>\\s*`);
  }

  /**
   * @param {string} header 
   * @param {string} [text]
   * @param  {...string} [rest]
   */
  log(header, text, ...rest) {
    return this.logger.log(header, text, ...rest);
  }

  /** @type {(error: Error) => void} */
  get catcher() {
    return (error) => {
      const text = error instanceof Error ? Util.logifyError(error) : error;
      this.logger.log('WARN', 'Caught an error', text);
    };
  }

  /**
   * @param {string} id
   */
  async loadManager(id) {
    const folder = path.join(this.dataDir, 'guilds');
    const manager = await Manager.create(id, folder);
    this.managers.set(id, manager);
  }

  /**
   * @param {string} id
   */
  async unloadManager(id) {
    const manager = this.managers.get(id);
    if (!manager) throw new Error('Cannot find manager with id ' + id);
    await manager.destroy();
  }

  getUserLevel(data) {
    let level = 0;

    if (data.message.guild) {
      if (data.message.member.hasPermission('ADMINISTRATOR')) level = 1;
      if (data.message.guild.owner.id === data.message.author.id) level = 2;
    } else if (this.trustedUsers.includes(data.message.author.id)) level = 3;

    if (this.owner === data.message.author.id) level = 10;

    return level;
  }

  /**
   * @param {Discord.Message} message 
   * @param {string} [prefixOverride]
   */
  parseCommand(message, prefixOverride) {
    if (message.author.bot) return null;
    const content = message.content.trim();
    const prefix = prefixOverride || this.prefix;
    if (!content.startsWith(prefix)) return null; // Dissallow whitespace between the prefix and command name

    if (/^\s+/.test(content.slice(prefix.length))) return null;
    let args = content.slice(prefix.length).trim().split(/\s+/g);
    if (args.length < 1) return null;
    const command = args.shift().toLowerCase();
    const argsText = content.slice(prefix.length + command.length).trim();
    return { message, command, args, argsText };
  }

  async destroy() {
    this.client.destroy();
    await Promise.all([
      this.logger.close(), 
      ...new Group(this.managers.values()).map(m => m.destroy())
    ]);
  }
}

/**
 * @typedef Config
 * @property {Discord.ClientOptions} [client]
 * @property {string} version
 * @property {string} token
 * @property {string} prefix
 * @property {string} owner
 * @property {string[]} trustedUsers
 * @property {string} dataDir
 */

/**
 * @typedef Options
 * @property {Logger} logger
 * @property {Group<Command>} commands
 * @property {Table<string, Manager>} managers
 * @property {Config} config
 */
