'use strict';
const path = require('path');
const { ws, write, read, mkdir, stat, exists } = require('../utils/fs.js');
const { makeLogEntry, savifyDate } = require('../utils/text.js');
const { mergeDefault } = require('../utils/obj.js');

class Logger {
  /**
   * @param {string} p 
   * @param {LoggerOptions} opts 
   */
  constructor(p, opts) {
    /** @type {LoggerOptions} */
    const options = mergeDefault(Logger.defaultOptions, opts);

    /** @type {string} */
    this.path = p;

    /** @type {boolean} */
    this.console = options.console;

    /** @type {string|null} */
    this.folder = options.folder;

    /** @type {number} */
    this.maxSize = options.maxSize;

    /** @type {WriteStream} */
    this.stream = ws.create(this.path);
  }

  /** @type {boolean} */
  get rotation() {
    return Boolean(this.folder);
  }

  /**
   * Creates an entry in this logger
   * @param {string} header
   * @param {string} [text]
   * @param {...string} [rest]
   */
  log(header, text, ...rest) {
    const writable = this.stream.writable;
    if (!writable) rest.push('[Not Written to LogFile]');
    const entry = makeLogEntry(header, text, ...rest);
    if (this.console) console.log(entry);
    if (writable) return this.stream.write(entry + '\n');
  }

  /**
   * Shuts down the logger
   * @returns {Promise<Void>}
   */
  async end() {
    await ws.end(this.stream);
  }

  /**
   * Checks log rotation and rotates logs if needed
   * @returns {Promise<Void>}
   */
  async checkRotation() {
    this.stream.cork();
    if (!await exists(this.folder))
      await mkdir(this.folder);

    let fileStats = await stat(this.path);
    let out = false;
    
    if (fileStats.size >= this.maxSize) {
      const contents = await read(this.path);
      await write(path.join(this.folder, savifyDate() + '.log'), contents);
      const entry = makeLogEntry('DATA', 'Log Rotated');
      if (this.console) console.log(entry);
      await write(this.path, entry + '\n');
      out = true;
    }

    this.stream.uncork();
    return out;
  }
}

Logger.defaultOptions = {
  console: false,
  folder: null,
  maxSize: 524288
};

module.exports = Logger;

/**
 * @typedef LoggerOptions
 * @property {boolean} [console]
 * @property {string|null} [folder]
 * @property {number} [maxSize]
 */

/**
 * @typedef {import('fs').WriteStream} WriteStream
 */
