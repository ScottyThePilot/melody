'use strict';
const { write, read } = require('../modules/utils/fs.js');
const { mergeDefault, get, set, has } = require('../modules/utils/object.js');

class Lazystore {
  /**
   * @param {string} path 
   * @param {LazystoreOptions} options 
   */
  constructor(p, options) {
    /** @type {string} */
    this.path = p;

    /** @type {LazystoreOptions} */
    this.options = mergeDefault(Lazystore.defaultOptions, options);

    /** @type {boolean} */
    this.ready = false;

    /** @type {object|null} */
    this.state = null;

    /** @type {boolean} */
    this.synced = false;
  }

  /**
   * Prepares the Lazystore for use
   * @returns {Promise<void>}
   */
  async init() {
    if (this.ready) throw new Error('Already Initialized');

    this.state = await this.resolveState();

    this.ready = true;
    this.synced = true;
  }

  /**
   * Retrieves a value from the Lazystore
   * @param {string|string[]} path
   * @returns {any}
   */
  get(p) {
    const out = get(this.state, p);
    this.touch();
    return out;
  }

  /**
   * Writes to a value in the Lazystore
   * @param {string|string[]} path 
   * @param {any} value 
   * @returns {void}
   */
  set(p, value) {
    set(this.state, p, value);
    this.touch();
  }

  /**
   * Determines whether a value exists at the given path
   * @param {string|string[]} path
   * @returns {boolean}
   */
  has(p) {
    const out = has(this.state, p);
    this.touch();
    return out;
  }

  /** @private */
  touch() {
    this.synced = false;
  }

  /**
   * Writes the current state to the Lazystore's state to disc
   * @param {boolean} force
   * @returns {Promise<Void>}
   */
  async write(force) {
    if (this.synced && !force) return false;

    await write(
      this.path,
      stringifyJSON(
        this.state,
        this.options.compact
      )
    );

    this.synced = true;
    return true;
  }

  /**
   * Resolves what value the state has on disc, or what
   * value it should have if no file exists at the given path
   * @returns {Promise<object>}
   * @private
   */
  async resolveState() {
    try {
      // Try to read data from the path
      const data = await read(this.path);
      try {
        return JSON.parse(data);
      } catch (e) {
        if (!this.options.wipeIfCorrupt)
          throw new Error('Unable to parse JSON');
      }
    } catch (e) {
      // Write the default state if the file couldn't be read
      await write(
        this.path, 
        stringifyJSON(
          this.options.defaultState,
          this.options.compact
        )
      );

      return deepClone(
        this.options.defaultState
      );
    }
  }
}

Lazystore.defaultOptions = {
  defaultState: {},
  wipeIfCorrupt: true,
  compact: false
};

module.exports = Lazystore;

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

/**
 * @typedef LazystoreOptions
 * @property {object} defaultState
 * @property {boolean} wipeIfCorrupt
 * @property {boolean} compact
 */
