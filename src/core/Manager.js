'use strict';
import Datastore from '../fs/Datastore.js';
import Logger from '../fs/Logger.js';
import Queue from '../utils/Queue.js';
import Util from '../utils/Util.js';
import path from 'path';
import fs from 'fs';

/**
 * @implements {import('../fs/Logger').LoggerAgent}
 * @implements {import('../fs/Datastore').DatastoreAgent}
 */
export default class Manager {
  /**
   * @param {string} id
   * @param {Logger} logger
   * @param {Datastore} store
   */
  constructor(id, logger, store) {
    /** @type {string} */
    this.id = id;
    /** @type {Logger} */
    this.logger = logger;
    /** @type {Datastore} */
    this.store = store;
    /** @type {Queue} */
    this.queue = new Queue();
  }

  /**
   * @param {string} id
   * @param {import('fs').PathLike} location
   */
  static async create(id, location) {
    const folder = path.join(location.toString(), id);
    await Util.suppressCode(fs.promises.mkdir(folder), 'EEXIST');

    const logger = await Logger.create(path.join(folder, 'latest.log'), {
      logsFolder: path.join(location.toString(), id, 'logs')
    });

    const store = await Datastore.create(path.join(folder, 'store.json'), {
      defaultState: Manager.defaultState
    });

    return new Manager(id, logger, store);
  }

  /**
   * @param {string} header
   * @param {string} [text]
   * @param {...string} [rest]
   * @returns {boolean}
   */
  log(header, text, ...rest) {
    return this.logger.log(header, text, ...rest);
  }

  /**
   * @param {string | string[]} p
   * @returns {any}
   */
  get(p) {
    return this.store.get(p);
  }

  /**
   * @param {string | string[]} p 
   * @param {any} value 
   */
  set(p, value) {
    this.store.set(p, value);
  }

  /**
   * @param {string | string[]} p
   * @returns {boolean}
   */
  has(p) {
    return this.store.has(p);
  }

  /**
   * @param {boolean} [force=false]
   * @returns {Promise<boolean>}
   */
  write(force = false) {
    return this.queue.wait(() => this.store.write(force));
  }

  async destroy(write = false) {
    await Promise.all([
      this.logger.close(),
      this.store.close(write)
    ]);
  }
}

/** @type {object} */
Manager.defaultState = {
  prefix: null,
  disabledCommands: [],
  loggingLevel: 0
};

// Level 0: None, Level 1: Edits/Deletions, Level 2: All Messages
