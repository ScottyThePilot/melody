'use strict';
import Util from '../utils/Util.js';
import path from 'path';
import fs from 'fs';

class LoggerBase {
  /**
   * @param {fs.PathLike} p 
   * @param {fs.WriteStream} stream 
   * @param {LoggerBaseOptions} [options] 
   */
  constructor(p, stream, options) {
    /** @type {LoggerBaseOptions} */
    this.options = Util.mergeDefault(LoggerBase.defaultOptions, options);
    /** @type {fs.WriteStream} */
    this.stream = stream;
    /** @type {fs.PathLike} */
    this.path = p;
    /** @type {boolean} */
    this.ready = false;
  }

  /**
   * @param {fs.PathLike} p 
   * @param {LoggerBaseOptions} options
   * @returns {Promise<LoggerBase>}
   */
  static async create(p, options) {
    const stream = fs.createWriteStream(p, { flags: 'a' });
    await Util.onceEvent(stream, 'ready');
    return await new LoggerBase(p, stream, options).init();
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
   * @param {string|Buffer} entry
   * @returns {boolean}
   */
  write(entry) {
    return this.stream.writable ? this.stream.write(entry + '\n') : true;
  }

  async close() {
    this.stream.end();
    this.ready = false;
    await Util.onceEvent(this.stream, 'finish');
  }
}

/** @type {LoggerBaseOptions} */
LoggerBase.defaultOptions = {
  logToConsole: false,
  logsFolder: null,
  maxFileSize: 524288
};

export default LoggerBase;

/**
 * @typedef LoggerBaseOptions
 * @property {boolean} logToConsole
 * @property {fs.PathLike | null} logsFolder
 * @property {number} maxFileSize
 */
