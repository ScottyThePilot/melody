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
    this.locked = false;
    
    var rotationCheckFinished = this.rotation ? this.checkRotation() : Promise.resolve();

    rotationCheckFinished.then(() => this.log('Begin Log'));
  }

  log(header, text = '', ...rest) {
    if (this.locked) rest.push('[Not Written to LogFile]');
    const h = header ? `[${header.toString().toUpperCase()}]` : '';
    const data = (text + (rest.length > 0 ? ':\n' + rest.map(e => '  ' + e).join('\n') : '')).trim();
    const l = Logger.logifyDate() + ': ' + h + (data.length > 0 ? ' ' + data : '');
    if (this.logToConsole) console.log(l);
    if (!this.locked) this.logStream.write(l + '\n');
  }

  end() {
    return new Promise((resolve, reject) => {
      this.locked = true;
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
    if (!fs.existsSync(this.logPath)) await mkdir(this.logPath);
    
    var fileStats = await stat(this.path);
    
    if (fileStats.size >= Logger.sizeThreshold) {
      this.locked = true;
      var contents = await readFile(this.path);
      var now = new Date();
      await writeFile(this.logPath + '/' + Logger.savifyDate(now) + '.log', contents);
      await writeFile(this.path, `${Logger.logifyDate(now)}: [DATA] Log Rotated`);
      this.locked = false;
      Logger.main.log('DATA', `Log ${this.path} Rotated into ${this.logPath}`);
    }
  }

  static msgFailCatcher(err) {
    Logger.main.log('WARN', Logger.logifyError(err));
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

  static logifyError(err) {
    return `${err.name}: ${err.message} (${err.code} ${err.path})`;
  }

  static getUptime(client) {
    let up = client.uptime;
    let upD = Math.floor(up / 8.64e+7);
    let upH = Math.floor(up / 3.6e+6) % 24;
    let upM = Math.floor(up / 60000) % 60;
    return [upD, upH, upM];
  }

  static phrasifyUptime(client) {
    let [upD, upH, upM] = Logger.getUptime(client);
    upD += (upD === 1 ? ' day' : ' days');
    upH += (upH === 1 ? ' hour' : ' hours');
    upM += (upM === 1 ? ' minute' : ' minutes');
    return upD + ', ' + upH + ' and ' + upM;
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