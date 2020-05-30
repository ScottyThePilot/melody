'use strict';
import Util from '../utils/Util.js';
import path from 'path';
import fs from 'fs';

class Logger {
  /**
   * @param {fs.PathLike} p 
   * @param {fs.WriteStream} stream 
   * @param {LoggerOptions} [options] 
   */
  constructor(p, stream, options) {
    /** @type {LoggerOptions} */
    this.options = Util.mergeDefault(Logger.defaultOptions, options);
    /** @type {fs.WriteStream} */
    this.stream = stream;
    /** @type {fs.PathLike} */
    this.path = p;
    /** @type {boolean} */
    this.ready = false;
  }

  /**
   * @param {fs.PathLike} p 
   * @param {LoggerOptions} options
   * @returns {Promise<Logger>}
   */
  static async create(p, options) {
    const stream = fs.createWriteStream(p, { flags: 'a' });
    await Util.onceEvent(stream, 'ready');
    return await new Logger(p, stream, options).init();
  }

  /**
   * @returns {Promise<this>}
   * @private
   */
  async init() {
    if (this.ready) throw new Error('Cannot initialize state more than once');

    if (this.options.logsFolder !== null) {
      await Util.suppressCode(fs.promises.mkdir(this.options.logsFolder, { recursive: true }), 'EEXIST');
      await this.rotate();
    }

    this.ready = true;
    return this;
  }

  /**
   * @returns {Promise<boolean>}
   */
  async rotate() {
    if (this.options.logsFolder === null) return false;

    const now = new Date();

    this.stream.cork();

    const handle = await fs.promises.open(this.path, 'r+');
    const { size } = await handle.stat();

    const rotate = size >= this.options.maxFileSize;
    if (rotate) {
      const folder = this.options.logsFolder.toString();
      const filepath = path.join(folder, Util.savifyDate(now) + '.log');
      
      const contents = await handle.readFile();
      await fs.promises.writeFile(filepath, contents, { flag: 'wx' });
      await handle.writeFile('');
    }

    await handle.close();

    this.stream.uncork();

    return rotate;
  }

  /**
   * @param {string} header
   * @param {string} [text]
   * @param {...string} [rest]
   * @returns {boolean}
   */
  log(header, text, ...rest) {
    const entry = Util.makeLogEntry(header, text, ...rest);
    if (this.options.logToConsole) console.log(getHeaderColor(header) + entry);
    return this.stream.writable ? this.stream.write(entry + '\n') : true;
  }

  async close() {
    this.stream.end();
    this.ready = false;
    await Util.onceEvent(this.stream, 'finish');
  }
}

/** @type {LoggerOptions} */
Logger.defaultOptions = {
  logToConsole: false,
  logsFolder: null,
  maxFileSize: 524288
};

Logger.colors = {
  NONE: '',
  RED: '\x1b[31m',
  GREEN: '\x1b[32m',
  YELLOW: '\x1b[33m',
  BLUE: '\x1b[34m',
  MAGENTA: '\x1b[35m',
  CYAN: '\x1b[36m'
};

Logger.headers = {
  [/err|error|fatal/i]: Logger.colors.RED,
  [/warn|alert/i]: Logger.colors.YELLOW,
  [/debug/i]: Logger.colors.BLUE
};

export default Logger;

/**
 * @param {string} header
 */
function getHeaderColor(header) {
  for (let color in Logger.colors) {
    const test = typeof color === 'string' ? header.toUpperCase() === color
      : color instanceof RegExp ? color.test(header) : false;
    if (test) return Logger.colors[color];
  }
  return '';
}

/**
 * @typedef LoggerOptions
 * @property {boolean} logToConsole
 * @property {fs.PathLike | null} logsFolder
 * @property {number} maxFileSize
 */
