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
    
    if (fileStats.size >= Logger.sizeThreshold) {
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

  static msgFailCatcher(err) {
    Logger.main.log('WARN', Logger.logifyError(err));
  }

  static makeLogEntry(header, text = '', ...rest) {
    const h = header ? `[${header.toString().toUpperCase()}]` : '';
    const data = (text + (rest.length > 0 ? ':\n' + rest.map(e => '  ' + e).join('\n') : '')).trim();
    return Logger.logifyDate() + ': ' + h + (data.length > 0 ? ' ' + data : '');
  }

  static logifyDate(date = new Date()) {
    let y = date.getFullYear();
    let d = pad(date.getDate());
    let m = pad(date.getMonth() + 1);
    let ms = (date.getMilliseconds() / 1000).toFixed(3).slice(2, 5);
    let t = date.toTimeString().split(' ');
    return `[${y}-${d}-${m}][${t[0]}.${ms}][${t[1]}]`;
  }

  static logifyDateLocale(date = new Date(), timeZone) {
    return '[' + date.toLocaleString('en-US', {
      year: 'numeric',
      month: 'numeric',
      day: 'numeric',
      hour: 'numeric',
      minute: '2-digit',
      timeZoneName: 'short',
      timeZone: timeZone
    }) + ']';
  }

  static parseLogDate(str) {
    const args = str.match(/\d+/g).slice(0, -1).map((e) => parseInt(e));
    return new Date(args[0], args[2] - 1, args[1], ...args.slice(3));
   }

  static replaceWithDateLocale(text, timeZone) {
    return text.replace(/\[\d{4}-\d{2}-\d{2}\]\[\d{2}:\d{2}:\d{2}.\d{3}\]\[.{8}\]/g, (m) => {
      return Logger.logifyDateLocale(Logger.parseLogDate(m), timeZone);
    });
  }

  static savifyDate(date = new Date()) {
    return Logger.logifyDate(date).slice(1, 25).replace(/[^0-9]+/g, '-');
  }

  static logifyUser(entity) {
    let user = entity.hasOwnProperty('user') ? entity.user : entity;
    return `${user.tag} (${user.id})` + (user.bot ? ' (BOT)' : '');
  }

  static logifyGuild(guild) {
    return Logger.logify(guild) + (guild.available ? '' : ' (Unavailable)');
  }

  static logifyError(err) {
    var info = err.code && err.path ? err.code + ' ' + err.path : err.code || err.path;
    return `${err.name || 'Error'}: ${err.message}` + (info ? ` (${info})` : '');
  }

  static logifyBytes(bytes) {
    return bytes < 1024 ? bytes.toFixed(3) + 'b'
      : bytes < 1048576 ? (bytes / 1024).toFixed(3) + 'kb'
      : bytes < 1073741824 ? (bytes / 1048576).toFixed(3) + 'mb'
      : (bytes / 1073741824).toFixed(3) + 'gb';
  }

  static stylizeAttachment(attachment) {
    return `${attachment.filename} (${Logger.logifyBytes(attachment.filesize)}): ${attachment.url}`;
  }
  
  static stylizeMetaData(message) {
    let c = message.embeds.length;
    let out = !c ? [] : [`[${c} Embed${c <= 1 ? '' : 's'}]`];
    return out.concat(message.attachments.array().map(Logger.stylizeAttachment));
  }

  static logify(obj) {
    return `${obj.name} (${obj.id})`;
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
    return ('' + str).replace(/["'\\\n\r\u2028\u2029]/g, function (ch) {
      switch (ch) {
        case '\"': return '\\\"';
        case '\'': return '\\\'';
        case '\\': return '\\\\';
        case '\n': return '\\n';
        case '\r': return '\\r';
        case '\u2028': return '\\u2028';
        case '\u2029': return '\\u2029';
      }
    });
  }

  static cleanContent(message) {
    return message.content
      .replace(/@(everyone|here)/g, '@\u200b$1')
      .replace(/<@!?[0-9]+>/g, input => {
        const id = input.replace(/<|!|>|@/g, '');
        if (message.channel.type === 'dm' || message.channel.type === 'group') {
          return message.client.users.has(id) ? `@${message.client.users.get(id).tag}` : input;
        }
  
        const user = message.client.users.get(id);
        if (user) return `@${user.tag}`;
        return input;
      })
      .replace(/<#[0-9]+>/g, input => {
        const channel = message.client.channels.get(input.replace(/<|#|>/g, ''));
        if (channel) return `#${channel.name}`;
        return input;
      })
      .replace(/<@&[0-9]+>/g, input => {
        if (message.channel.type === 'dm' || message.channel.type === 'group') return input;
        const role = message.guild.roles.get(input.replace(/<|@|>|&/g, ''));
        if (role) return `@${role.name}`;
        return input;
      });
  }
}

Logger.sizeThreshold = 1048576; // File threshold in bytes

module.exports = Logger;
