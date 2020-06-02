'use strict';
import Datastore from '../fs/Datastore.js';
import Register from '../fs/Register.js';
import Queue from '../utils/Queue.js';
import Util from '../utils/Util.js';
import path from 'path';
import fs from 'fs';

export default class Manager {
  /**
   * @param {string} id
   * @param {Register} register
   * @param {Datastore} store
   */
  constructor(id, register, store) {
    /** @type {string} */
    this.id = id;
    /** @type {Register} */
    this.register = register;
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

    const register = await Register.create(path.join(folder, 'register.db'), {
      logsFolder: path.join(folder, 'logs')
    });

    const store = await Datastore.create(path.join(folder, 'store.json'), {
      defaultState: Manager.defaultState
    });

    return new Manager(id, register, store);
  }

  async destroy(write = false) {
    await Promise.all([
      this.register.close(),
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
