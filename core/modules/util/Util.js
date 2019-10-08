'use strict';
const moment = require('moment');

const util = {
  shuffle(array) {
    if (!Array.isArray(array)) throw new TypeError('Expected an array');
    var arr = array.slice(0);
    var currentIndex = arr.length, temporaryValue, randomIndex;

    while (0 !== currentIndex) {
      randomIndex = Math.floor(Math.random() * currentIndex);
      currentIndex -= 1;

      temporaryValue = arr[currentIndex];
      arr[currentIndex] = arr[randomIndex];
      arr[randomIndex] = temporaryValue;
    }

    return arr;
  },

  mergeDefault(def, given) {
    if (!given) return def;
    for (const key in def) {
      if (!{}.hasOwnProperty.call(given, key)) {
        given[key] = def[key];
      } else if (given[key] === Object(given[key])) {
        given[key] = Util.mergeDefault(def[key], given[key]);
      }
    }

    return given;
  },

  format(str, ...replacers) {
    if (typeof str !== 'string') throw new TypeError('Expected a string');
    return str.replace(/{(\d+)}/g, function(match, number) {
      return typeof replacers[number] !== 'undefined' ? replacers[number] : match;
    });
  },

  capFirst(str) {
    return ''.charAt.call(str, 0).toUpperCase() + ''.slice.call(str, 1).toLowerCase();
  },

  async asyncForEach(array, callback) {
    for (let i = 0; i < array.length; i ++) {
      await callback(array[i], i, array);
    }
  },

  wait(ms) {
    return new Promise((resolve) => setTimeout(resolve, ms));
  },

  logifyDate(date) {
    return '[' + moment(date).format('YYYY-MM-DD HH:mm:ss.SSS [UTC]ZZ') + ']';
  },

  savifyDate(date) {
    return Util.logifyDate(date).slice(1, 24).replace(/[^0-9]+/g, '_'); 
  },

  makeLogEntry(header, text = '', ...rest) {
    const h = header ? `[${('' + header).toUpperCase()}] ` : '';
    const r = rest.length ? ':\n' + rest.map(e => '  ' + e).join('\n') : '';
    const data = text.trim() + r.trim();
    return Util.logifyDate() + (h.length || data.length ? ': ' : '') + h + data;
  },

  logifyUser(entity) {
    let user = entity.hasOwnProperty('user') ? entity.user : entity;
    return `${user.tag} (${user.id})` + (user.bot ? ' (BOT)' : '');
  },

  logifyGuild(guild) {
    return Util.logify(guild) + (guild.available ? '' : ' (Unavailable)');
  },

  logifyError(err) {
    var info = err.code && err.path ? err.code + ' ' + err.path : err.code || err.path;
    return `${err.name || 'Error'}: ${err.message}` + (info ? ` (${info})` : '');
  },

  logify(obj) {
    return `${obj.name} (${obj.id})`;
  },

  stylizeAttachment(attachment) {
    return `${attachment.filename} (${Util.formatBytes(attachment.filesize)}): ${attachment.url}`;
  },
  
  stylizeMetaData(message) {
    let c = message.embeds.length;
    let out = !c ? [] : [`[${c} Embed${c <= 1 ? '' : 's'}]`];
    return [out, ...message.attachments.array().map(Util.stylizeAttachment)];
  },

  formatTime(uptime, short = false) {
    if (uptime < 60000) return short ? '<1m' : 'less than a minute';
    const upD = Math.floor(uptime / 8.64e+7);
    const upH = Math.floor(uptime / 3.6e+6) % 24;
    const upM = Math.floor(uptime / 60000) % 60;
    const upDstr = upD + (short ? 'd' : ' day' + (upD === 1 ? '' : 's'));
    const upHstr = upH + (short ? 'h' : ' hour' + (upH === 1 ? '' : 's'));
    const upMstr = upM + (short ? 'm' : ' minute' + (upM === 1 ? '' : 's'));
    return (upD ? upDstr + ', ' : '') + (upD || upH ? upHstr + ' and ' : '') + upMstr;
  },

  formatBytes(bytes) {
    return bytes < 1024 ? bytes.toFixed(3) + 'b'
      : bytes < 1048576 ? (bytes / 1024).toFixed(3) + 'kb'
      : bytes < 1073741824 ? (bytes / 1048576).toFixed(3) + 'mb'
      : (bytes / 1073741824).toFixed(3) + 'gb';
  },

  escape(str) {
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
  },

  cleanContent(message) {
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
} = module.exports;
