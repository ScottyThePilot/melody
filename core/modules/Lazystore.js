'use strict';
const { write, read, exists } = require('./util/fswrapper.js');
const { get, set, has } = require('./util/mut.js');
const { mergeDefault } = require('./util/util.js');

class Lazystore {
  constructor(path) {
    this.path = path;
    this.options = mergeDefault(Lazystore.defaultOptions, options);
    this.ready = false;
    this.state = null;
    this.synced = false;

    this.init();
  }

  async init() {
    if (this.ready) throw new Error('Already Initialized');

    if (exists(this.path)) {
      const parsed = tryParseJSON(await read(this.path), () => {
        if (this.options.wipeIfCorrupt) {
          write(stringifyJSON(this.options.defaultData, this.options.compact));
          return deepClone(this.options.defaultData);
        }
        throw new Error('Unable to parse JSON');
      });
    } else {

    }

    this.ready = true;
    this.synced = true;
  }

  get(path) {
    //
    this.touch();
  }

  set(path, value) {
    //
    this.touch();
  }

  has(path) {
    //
    this.touch();
  }

  touch() {
    //
    this.synced = false;
  }

  async write() {
    //
    this.synced = true;
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
    await write(stringifyJSON(this.options.defaultData, this.options.compact));
    return deepClone(this.options.defaultData);
  }
}

Lazystore.defaultOptions = {
  defaultData: {},
  wipeIfCorrupt: true,
  compact: false
};

module.exports = Lazystore;

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
