'use strict';
const { write, read, exists } = require('../modules/fswrapper.js');
const { get, set, has } = require('../modules/mut.js');
const { mergeDefault } = require('../modules/util.js');

class Lazystore {
  constructor(path, options) {
    this.path = path;
    this.options = mergeDefault(Lazystore.defaultOptions, options);
    this.ready = false;
    this.state = null;
    this.synced = false;
  }

  async init() {
    if (this.ready) throw new Error('Already Initialized');

    this.state = await this.resolveState();

    this.ready = true;
    this.synced = true;
  }

  get(path) {
    const out = get(this.state, path);
    this.touch();
    return out;
  }

  set(path, value) {
    set(this.state, path, value);
    this.touch();
  }

  has(path) {
    const out = has(this.state, path);
    this.touch();
    return out;
  }

  touch() {
    this.synced = false;
  }

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

  async resolveState() {
    if (exists(this.path)) {
      const data = await read(this.path);
      try {
        return JSON.parse(data);
      } catch (e) {
        if (!this.options.wipeIfCorrupt)
          throw new Error('Unable to parse JSON');
      }
    }

    await write(
      this.path, 
      stringifyJSON(
        this.options.defaultData,
        this.options.compact
      )
    );
    return deepClone(this.options.defaultData);
  }
}

Lazystore.defaultOptions = {
  defaultData: {},
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
