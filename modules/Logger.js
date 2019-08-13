'use strict';
const fs = require('fs');
const promisify = require('util').promisify;

const writeFile = promisify(fs.writeFile);
const readFile = promisify(fs.readFile);
const mkdir = promisify(fs.mkdir);
const stat = promisify(fs.stat);

class Logger {
  constructor(path, options = {}) {
    this.path = path;
    this.logToConsole = {}.hasOwnProperty.call(options, 'logToConsole') ? options.logToConsole : false;
    this.logPath = {}.hasOwnProperty.call(options, 'logPath') ? options.logPath : null;
    this.rotation = Boolean(this.logPath);
    this.logStream = fs.createWriteStream(this.path, { flags: 'a' });
    this.postponed = false;
    if (this.rotation) {
      this.checkRotation().then(() => {
        this.log('Begin Log');
      });
    } else {
      this.log('Begin Log');
    }
    
  }

  log(header, text, ...rest) {
    const h = header ? ` [${header.toString().toUpperCase()}]` : '';
    const data = text + (rest.length > 0 ? ':\n' + rest.map(e => '  ' + e).join('\n') : '');
    const l = Logger.logifyDate() + ':' + h + (data !== 'undefined' ? ' ' + data : '');
    if (this.logToConsole) console.log(l);
    if (this.postponed) return;
    this.logStream.write(l + '\n');
  }

  end() {
    var stream = this.logStream;
    return new Promise((resolve, reject) => {
      stream.end((err) => {
        if (err) {
          reject(err);
        } else {
          resolve();
        }
      });
    });
  }

  async checkRotation() {
    if (!fs.existsSync(this.logPath)) await mkdir(this.logPath);
    
    var fileStats = await stat(this.path);
    console.log(fileStats);
    if (fileStats.size >= Logger.sizeThreshold) {
      this.postponed = true;
      var contents = await readFile(this.path);
      var now = new Date();
      await writeFile(this.logPath + '/' + Logger.savifyDate(now) + '.log', contents);
      await writeFile(this.path, Logger.logifyDate(now) + ': [INFO] Log Rotated\n');
      this.postponed = false;
    }
  }

  static msgFailCatcher() {
    Logger.main.log('MSG', 'Failed to send a message');
  }

  static logifyDate(date = new Date()) {
    return '[' + Date.prototype.toISOString.call(date).replace(/T/, '][').replace(/Z/, ']');
  }

  static savifyDate(date = new Date()) {
    return Date.prototype.toISOString.call(date).slice(0, 19).replace(/[^0-9]/g, '_');
  }

  static logifyUser(entity) {
    var user = entity.hasOwnProperty('user') ? entity.user : entity;
    return `${user.tag} (${user.id})`;
  }

  static logifyGuild(guild) {
    return `${guild.name} (${guild.id})` + (guild.available ? '' : ' (Unavailable)');
  }

  static escape(str) {
    return ''.replace.call(''.replace.call(str, /\n/g, '\\n'), /[\f\r\t\v]/g, '');
  }
}

Logger.main = new Logger('./data/main.log', {
  logToConsole: true,
  logPath: './data/logs'
});

Logger.sizeThreshold = 1048576; // File threshold in bytes

module.exports = Logger;