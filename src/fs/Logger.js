'use strict';
import LoggerBase from './LoggerBase.js';
import Util from '../utils/Util.js';
import fs from 'fs';

class Logger extends LoggerBase {
  /**
   * @param {fs.PathLike} p 
   * @param {fs.WriteStream} stream 
   * @param {LoggerBaseOptions} [options] 
   */
  constructor(p, stream, options) {
    super(p, stream, options);
    this.options.fileExtension = '.log';
  }

  /**
   * @param {fs.PathLike} p 
   * @param {import('./LoggerBase.js').LoggerBaseOptions} options
   * @returns {Promise<Logger>}
   */
  static async create(p, options) {
    const stream = fs.createWriteStream(p, { flags: 'a' });
    await Util.onceEvent(stream, 'ready');
    return await new Logger(p, stream, options).init();
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
    return this.writeEntry(entry + '\n');
  }
}

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
 * @property {boolean} wipeOnLoad
 * @property {number} maxFileSize
 */
