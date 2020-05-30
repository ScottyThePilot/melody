'use strict';
import Util from '../utils/Util.js';
import fs from 'fs';

/**
 * @implements {DatastoreAgent}
 */
class Datastore {
  /**
   * @param {import('fs').PathLike} p
   * @param {import('fs').promises.FileHandle} handle
   * @param {DatastoreOptions} [options]
   */
  constructor(p, handle, options) {
    /** @type {DatastoreOptions} */
    this.options = Util.mergeDefault(Datastore.defaultOptions, options);
    /** @type {import('fs').promises.FileHandle} */
    this.handle = handle;
    /** @type {import('fs').PathLike} */
    this.path = p;
    /** @type {object | null} */
    this.state = null;
    /** @type {boolean} */
    this.ready = false;
    /** @type {boolean} */
    this.synced = false;
  }

  /**
   * @param {import('fs').PathLike} p 
   * @param {DatastoreOptions} [options]
   * @returns {Promise<Datastore>}
   */
  static async create(p, options) {
    return await new Datastore(p, await fs.promises.open(p, 'w+'), options).init();
  }

  /**
   * @returns {Promise<this>}
   * @private
   */
  async init() {
    if (this.ready) throw new Error('Cannot initialize state more than once');

    this.state = await this.resolveState();

    this.ready = true;
    this.synced = true;

    return this;
  }

  /**
   * @param {string | string[]} p
   * @returns {any}
   */
  get(p) {
    if (!this.ready) throw new Error('Unable to read/modify state');
    const out = Util.get(this.state, p);
    this.synced = false;
    return out;
  }

  /**
   * @param {string | string[]} p 
   * @param {any} value 
   */
  set(p, value) {
    if (!this.ready) throw new Error('Unable to read/modify state');
    Util.set(this.state, p, value);
    this.synced = false;
  }

  /**
   * @param {string | string[]} p
   * @returns {boolean}
   */
  has(p) {
    if (!this.ready) throw new Error('Unable to read/modify state');
    const out = Util.has(this.state, p);
    this.synced = false;
    return out;
  }

  /**
   * @param {boolean} [force=false]
   * @returns {Promise<boolean>}
   */
  async write(force = false) {
    if (!this.ready) throw new Error('Cannot write state to disk');
    if (this.synced && !force) return false;

    await this.handle.writeFile(this.stringify(this.state), { flag: 'r+' });

    this.synced = true;
    return true;
  }

  /**
   * @param {boolean} [write=false]
   */
  async close(write = false) {
    if (!this.ready) throw new Error('Unable to destroy datastore');

    if (write) await this.write(true);
    await this.handle.close();

    this.ready = false;
    this.synced = false;
    this.state = null;
  }

  /**
   * @returns {Promise<object>}
   * @private
   */
  async resolveState() {
    const wipe = this.options.wipeIfCorrupt;
    let data = await this.handle.readFile({ flag: 'r+' });
    try {
      data = parseJSON(data);
    } catch (e) {
      if (wipe) {
        data = this.stringify(this.options.defaultState);
        await this.handle.writeFile(data, { flag: 'w+' });
        data = parseJSON(data);
      } else throw e;
    } finally {
      return data;
    }
  }

  /**
   * @param {any} value
   * @returns {string}
   * @private
   */
  stringify(value) {
    return JSON.stringify(value, null, this.options.compact ? 0 : 2);
  }
}

/** @type {DatastoreOptions} */
Datastore.defaultOptions = {
  defaultState: {},
  wipeIfCorrupt: true,
  compact: false
};

export default Datastore;

/**
 * @param {string | Buffer} text
 * @returns {any}
 */
function parseJSON(text) {
  return JSON.parse(text.toString());
}

/**
 * @typedef DatastoreOptions
 * @property {object} defaultState
 * @property {boolean} wipeIfCorrupt
 * @property {boolean} compact
 */

/**
 * @typedef DatastoreAgent
 * @property {(p: string | string[]) => any} get
 * @property {(p: string | string[], value: any) => void} set
 * @property {(p: string | string[]) => boolean} has
 * @property {(force: boolean) => Promise<boolean>} write
 */
