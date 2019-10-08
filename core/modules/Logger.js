'use strict';
const { createWriteStream } = require('fs');
const { write, read, mkdir, stat, exists } = require('./util/fswrapper.js');

function pad(val) {
  return val < 10 ? '0' + val : val + '';
}

class Logger {
  constructor(path, options = {}) {
    this.path = path;
    this.logToConsole = {}.hasOwnProperty.call(options, 'logToConsole') ? options.logToConsole : false;
    this.logPath = {}.hasOwnProperty.call(options, 'logPath') ? options.logPath : null;
    this.rotation = Boolean(this.logPath);
    this.logStream = createWriteStream(this.path, { flags: 'a' });
    this.sizeThreshold = options.sizeThreshold || Logger.defaultSizeThreshold;
    
    if (this.rotation) {
      this.checkRotation();
    } else {
      this.log('Begin Log');
    }
  }

  log(header, text = '', ...rest) {
    const writable = this.logStream.writable;
    if (!writable) rest.push('[Not Written to LogFile]');
    const entry = Logger.makeLogEntry(header, text, ...rest);
    if (this.logToConsole) console.log(entry);
    if (writable) return this.logStream.write(entry + '\n');
  }

  end() {
    return new Promise((resolve, reject) => {
      this.logStream.end((err) => {
        if (err) {
          reject(err);
        } else {
          resolve();
        }
      });
    });
  }

  async checkRotation() {
    this.logStream.cork();
    if (!exists(this.logPath)) await mkdir(this.logPath);
    
    let fileStats = await stat(this.path);
    
    if (fileStats.size >= this.sizeThreshold) {
      const contents = await read(this.path);

      await write(this.logPath + '/' + Logger.savifyDate() + '.log', contents);

      const entry1 = Logger.makeLogEntry('DATA', 'Log Rotated');
      const entry2 = Logger.makeLogEntry('Begin Log');

      if (this.logToConsole) {
        console.log(entry1);
        console.log(entry2);
      }

      await write(this.path, entry1 + '\n' + entry2 + '\n');

      Logger.main.log('DATA', `Log ${this.path} Rotated into ${this.logPath} and data written`);
    }

    process.nextTick(() => this.logStream.uncork());
  }
}

Logger.defaultSizeThreshold = 1048576; // File threshold in bytes

module.exports = Logger;
