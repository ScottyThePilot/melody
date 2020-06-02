'use strict';
import LoggerBase from './LoggerBase.js';
import Util from '../utils/Util.js';
import fs from 'fs';

class Register extends LoggerBase {
  /**
   * @param {fs.PathLike} p 
   * @param {fs.WriteStream} stream 
   * @param {LoggerBaseOptions} [options] 
   */
  constructor(p, stream, options) {
    super(p, stream, options);
    /** @type {fs.PathLike} */
    this.path = p;
    this.options.fileExtension = '.db';
  }

  /**
   * @param {fs.PathLike} p 
   * @param {import('./LoggerBase.js').LoggerBaseOptions} options
   * @returns {Promise<Register>}
   */
  static async create(p, options) {
    const stream = fs.createWriteStream(p, { flags: 'a' });
    await Util.onceEvent(stream, 'ready');
    return await new Register(p, stream, options).init();
  }

  /**
   * @param {object} entry
   * @returns {boolean}
   */
  write(entry) {
    return super.writeEntry(JSON.stringify(entry));
  }

  /**
   * @param {number} [count]
   * @returns {Promise<object[]>}
   */
  async read(count) {
    const data = await fs.promises.readFile(this.path);
    let array = data.toString().split(/\n/).reverse();
    if (count !== undefined) array = array.slice(0, count);
    return array.map((e) => JSON.parse(e));
  }
}

export default Register;
