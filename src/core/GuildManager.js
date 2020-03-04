'use strict';
const path = require('path');
const Logger = require('./Logger.js');
const Lazystore = require('./Lazystore.js');
const Queue = require('./Queue.js');
const { mkdir, exists } = require('../utils/fs.js');

class GuildManager {
  /**
   * @param {string} id 
   * @param {Logger} logger 
   * @param {Lazystore} store 
   */
  constructor(id, logger, store) {
    /** @type {string} */
    this.id = id;
    
    /** @type {Logger} */
    this.logger = logger;
    
    /** @type {Lazystore} */
    this.store = store;
    
    /** @type {Queue} */
    this.queue = new Queue();
  }

  /**
   * Loads a new GuildManager instance for a given server
   * @param {string} id 
   * @param {string} location 
   * @param {object} defaultConfig 
   * @returns {Promise<GuildManager>}
   */
  static async load(id, location, defaultConfig) {
    const folder = path.join(location, id);
    if (!await exists(folder)) await mkdir(folder);
    
    const logger = new Logger(path.join(folder, 'latest.log'), {
      folder: path.join(location, id, 'logs')
    });

    const store = new Lazystore(path.join(folder, 'store.json'), {
      defaultData: defaultConfig
    });

    await store.init();

    return new GuildManager(id, logger, store);
  }

  /**
   * Creates an entry in this GuildManager's logger
   * @param {string} header
   * @param {string} text
   * @param {...string} rest
   */
  log(header, text, ...rest) {
    this.logger.log(header, text, ...rest);
  }

  /**
   * @param {string|string[]} p
   * @returns {any}
   */
  get(p) {
    return this.store.get(p);
  }

  /**
   * @param {string|string[]} p
   * @param {any} value 
   * @returns {void}
   */
  set(p, value) {
    this.store.set(p, value);
  }

  /**
   * @param {string|string[]} p
   * @returns {boolean}
   */
  has(p) {
    return this.store.has(p);
  }

  /**
   * Orders this manager to write its state to disc
   * @returns {Promise<void>}
   */
  write() {
    return this.queue.pushPromise(() => this.store.write());
  }
}

module.exports = GuildManager;
