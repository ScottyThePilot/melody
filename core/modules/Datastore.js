/* jshint esversion: 8, -W014 */
'use strict';
const { write, read, exists } = require('./util/fswrapper.js');

/**
 * A class to manage the execution of async functions or
 * Promise-generating functions. functions given to it
 * will be executed one at a time.
 * @private
 */
class Sequencer {
  constructor() {
    this.working = false;
    this.current = Promise.resolve();
    this.items = [];
  }

  /**
   * Adds a function to the queue to be executed.
   * @param fn {Function<Promise>} The function to be added
   */
  push(fn) {
    return new Promise((resolve, reject) => {
      fn.resolve = resolve;
      fn.reject = reject;
      this.items.push(fn);
      if (!this.working) {
        this.working = true;
        this.next();
      }
    });
  }

  /**
   * Used internally to move from function to function
   * @private
   */
  next() {
    if (this.items.length) {
      this.working = true;
      let next = this.items.shift();
      let current = wait(next());
      this.current = current;
      current.then((out) => {
        next.resolve(out);
        this.next();
      }).catch((reason) => {
        if (next.reject(reason)) this.next();
      });
    } else {
      this.working = false;
    }
  }
}

class ActionBatch {
  constructor(datastore, actions = []) {
    this.datastore = datastore;
    this.actions = actions;
    this.done = false;
  }

  get(path) {
    if (this.done) throw new Error();
    this.actions.push({ type: 'get', path });
    return this;
  }

  set(path, value) {
    if (this.done) throw new Error();
    this.actions.push({ type: 'set', path, value });
    return this;
  }

  has(path) {
    if (this.done) throw new Error();
    this.actions.push({ type: 'has', path });
    return this;
  }

  edit(callback) {
    if (this.done) throw new Error();
    this.actions.push({ type: 'edit', callback });
    return this;
  }

  async go() {
    return this.datastore.sequencer.push(async () => {
      const ds = this.datastore;
      let data = await ds.resolveData(Symbol.for('wipe'));
      let shouldWrite = data === Symbol.for('wipe');

      if (shouldWrite) data = deepClone(ds.options.defaultData);

      const operations = {
        get(action) {
          return get(data, action.path);
        },
        set(action) {
          shouldWrite = true;
          return set(data, action.path, action.value);
        },
        has(action) {
          return has(data, action.path);
        },
        edit(action) {
          shouldWrite = true;
          return action.callback(data);
        }
      };

      this.actions.forEach((action) => operations[action.type](action));

      if (shouldWrite) {
        await write(
          ds.path,
          stringifyJSON(data, ds.options.compact)
        );
      }

      return data;
    });
  }

  static async batchFunction(ds, fn) {
    let data = await ds.resolveData(Symbol.for('wipe'));
    let shouldWrite = data === Symbol.for('wipe');

    if (shouldWrite) data = deepClone(ds.options.defaultData);

    await fn({
      get(path) {
        return get(data, path);
      },
      set(path, value) {
        shouldWrite = true;
        return set(data, path, value);
      },
      has(path) {
        return has(data, path);
      },
      edit(callback) {
        shouldWrite = true;
        return callback(data);
      }
    });

    if (shouldWrite) {
      await write(
        ds.path,
        stringifyJSON(data, ds.options.compact)
      );
    }

    return data;
  }
}

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
    this.sequencer = new Sequencer();

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
      await this.sequencer.push(async () => {
        await write(
          this.path,
          stringifyJSON(this.options.defaultData, this.options.compact)
        );
      });
    }

    if (this.options.persistence) {
      await this.sequencer.push(async () => {
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
    return this.sequencer.push(async () => {
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
    return this.sequencer.push(async () => {
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
    return this.sequencer.push(async () => {
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
    return this.sequencer.push(async () => {
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
   * Allows you to execute multiple operations on a datastore without 
   * extra read and write operations.
   * @param {Array|Function} obj 
   * @returns {ActionBatch}
   */
  batch(obj) {
    if (typeof obj === 'function') 
      return this.sequencer.push(
        () => ActionBatch.batchFunction(this, obj)
      );
    return new ActionBatch(obj);
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

function wait(val) {
  return val instanceof Promise ? val : Promise.resolve(val);
}

function tryParseJSON(str, def) {
  try {
    return JSON.parse(str);
  } catch (e) {
    if (typeof def === 'function') def();
    return def;
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

function validate(obj, path) {
  if (obj === null || obj === undefined) throw new Error('Invalid Object: ' + obj);
  if (path === undefined || !(Array.isArray(path) ? path : '' + path).length)
    return null;
  const a = Array.isArray(path)
    ? path
    : path
      .replace(/\[(\w+)\]/g, '.$1')
      .replace(/^\./, '')
      .split('.');
  if (a.some(key => !/^(?:[0-9]|[a-zA-Z_$][a-zA-Z_$0-9\-]*)$/.test(key)))
    throw new Error('Invalid Path');
  return a;
}

function get(obj, path) {
  const a = validate(obj, path);
  if (a === null) return obj;

  for (let key of a) {
    if (key in obj) {
      obj = obj[key];
    } else {
      return;
    }
  }

  return obj;
}

function set(obj, path, value) {
  const a = validate(obj, path);
  if (a === null) return;

  while (a.length > 1) {
    let key = a.shift();
    let v = obj[key];
    obj = obj[key] =
      typeof v === 'object' && v !== null
        ? v
        : isNaN(a[0])
          ? {}
          : [];
  }

  obj[a[0]] = value;
}

function has(obj, path) {
  const a = validate(obj, path);
  if (a === null) return true;

  for (let key of a) {
    if (key in obj) {
      obj = obj[key];
    } else {
      return false;
    }
  }

  return true;
}