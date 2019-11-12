'use strict';
const { write, read, exists } = require('./util/fswrapper.js');
const { get, set, has } = require('./util/mut.js');
const { mergeDefault } = require('./util/util.js');
const Queue = require('./Queue.js');

const noPersistenceError = 'Persistence is not enabled';

class Datastore {
  /**
   * Creates a new Datastore object
   * @param {String} path The string path to the file
   * @param {Object} options 
   */
  constructor(path, options) {
    this.path = path;
    this.options = mergeDefault(Datastore.defaultOptions, options);
    this.ready = false;
    this.queue = new Queue();

    if (this.options.persistence)
      this.persistentState = null;

    this.init();
  }

  /**
   * Initializes the datastore, creating the file and writing default
   * data if it does not exist, and stores the persistentState if
   * persistence is enabled. Called upon object instatiation.
   * @private
   */
  async init() {
    if (this.ready) throw new Error('Already Initialized');

    if (!exists(this.path)) {
      await this.queue.push(async () => {
        await write(
          this.path,
          stringifyJSON(this.options.defaultData, this.options.compact)
        );
      });
    }

    if (this.options.persistence) {
      await this.queue.push(async () => {
        let data = await this.resolveDataWrite(true);
        this.persistentState = get(data);
      });
    }

    this.ready = true;
  }

  /**
   * Gets the value at the given path. Returns undefined if the
   * property does not exist.
   * @param {String|Array} path The string path to the property
   */
  get(path) {
    return this.queue.push(async () => {
      let data = await this.resolveDataWrite();
      return get(data, path);
    });
  }

  getSync(path) {
    if (!this.options.persistence)
      throw new Error(noPersistenceError);
    return get(this.persistentState, path);
  }

  /**
   * Sets the value at the given path. Creates objects or arrays
   * to the path if they do not exist.
   * @param {String|Array} path The string path to the property
   * @param {*} value The value to set
   */
  set(path, value) {
    return this.queue.push(async () => {
      let data = await this.resolveData();
      set(data, path, value);

      await write(
        this.path,
        stringifyJSON(data, this.options.compact)
      );

      return data;
    });
  }

  /**
   * Returns a boolean indicating whether a property exists at the
   * given path.
   * @param {String|Array} path The string path to the property
   * @returns {Boolean}
   */
  has(path) {
    return this.queue.push(async () => {
      let data = await this.resolveDataWrite();
      return has(data, path);
    });
  }

  hasSync(path) {
    if (!this.options.persistence)
      throw new Error(noPersistenceError);
    return has(this.persistentState, path);
  }

  /**
   * Allows custom changes to be made to the entire file.
   * @param {Function} callback A function to be used to transform
   *   the file's data. This function takes one argument, `data`
   */
  edit(callback) {
    return this.queue.push(async () => {
      let data = await this.resolveData();
      callback(data);

      await write(
        this.path,
        stringifyJSON(data, this.options.compact)
      );

      return data;
    });
  }

  /**
   * Retrieves data from the file, overwriting it if it is corrupt.
   * @private
   */
  async resolveDataWrite(ignorePersistence) {
    let data = await this.resolveData(Symbol.for('wipe'), ignorePersistence);

    if (data === Symbol.for('wipe')) {
      data = deepClone(this.options.defaultData);
      await write(
        this.path,
        stringifyJSON(data, this.options.compact)
      );
    }

    return data;
  }

  /**
   * Retrieves data from the file, ignoring it if it is corrupt.
   * @private
   */
  async resolveData(def, ignorePersistence) {
    def = def === undefined 
      ? deepClone(this.options.defaultData)
      : def;
    return this.options.persistence && !ignorePersistence
      ? this.persistentState
      : tryParseJSON(
        await read(this.path),
        this.options.wipeIfCorrupt
          ? def
          : () => { throw new Error('Unable to parse JSON'); }
      );
  }
}

Datastore.defaultOptions = {
  persistence: false,
  defaultData: {},
  wipeIfCorrupt: true,
  compact: false
};

module.exports = Datastore;

function tryParseJSON(str, def) {
  try {
    return JSON.parse(str);
  } catch (e) {
    return typeof def === 'function' ? def() : def;
  }
}

function stringifyJSON(obj, compact) {
  return JSON.stringify(obj, null, compact ? 0 : 2);
}

function deepClone(obj) {
  if (!obj) return obj;
  obj = Array.isArray(obj)
    ? [].slice.call(obj)
    : Object.assign({}, obj);

  for (const key in obj) {
    if (obj[key] === Object(obj[key])) {
      obj[key] = deepClone(obj[key]);
    }
  }

  return obj;
}
