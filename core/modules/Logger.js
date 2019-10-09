'use strict';
const { createWriteStream } = require('fs');
const path = require('path');
const { write, read, mkdir, stat, exists } = require('./util/fswrapper.js');
const util = require('./util/util.js');

class Logger {
  constructor(path, options = {}) {
    this.path = path;
    this.logToConsole = {}.hasOwnProperty.call(options, 'logToConsole') ? options.logToConsole : false;
    this.logPath = {}.hasOwnProperty.call(options, 'logPath') ? options.logPath : null;
    this.rotation = Boolean(this.logPath);
    this.logStream = createWriteStream(this.path, { flags: 'a' });
    this.sizeThreshold = options.sizeThreshold || Logger.defaultSizeThreshold;
  }

  log(header, text, ...rest) {
    const writable = this.logStream.writable;
    if (!writable) rest.push('[Not Written to LogFile]');
    const entry = util.makeLogEntry(header, text, ...rest);
    if (this.logToConsole) console.log(entry);
    if (writable) return this.logStream.write(entry + '\n');
  }

  end() {
    return new Promise((resolve, reject) => {
      this.logStream.end((err) => err ? reject(err) : resolve());
    });
  }

  async checkRotation(logger) {
    this.logStream.cork();
    if (!exists(this.logPath)) await mkdir(this.logPath);
    
    let fileStats = await stat(this.path);
    
    if (fileStats.size >= this.sizeThreshold) {
      const contents = await read(this.path);

      await write(path.join(this.logPath, util.savifyDate() + '.log'), contents);

      const entry = util.makeLogEntry('DATA', 'Log Rotated');

      if (this.logToConsole) console.log(entry);

      await write(this.path, entry + '\n');
      
      if (logger) logger.log('DATA', `Log ${this.path} Rotated into ${this.logPath}`);
    }

    process.nextTick(() => this.logStream.uncork());
  }
}

Logger.defaultSizeThreshold = 1048576; // File threshold in bytes

module.exports = Logger;
