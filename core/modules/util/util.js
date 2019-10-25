'use strict';
const moment = require('moment');
const charmap = require('./charmap.json');

function shuffle(array) {
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
}

function mergeDefault(def, given) {
  if (!given) return def;
  for (const key in def) {
    if (!{}.hasOwnProperty.call(given, key)) {
      given[key] = def[key];
    } else if (given[key] === Object(given[key])) {
      given[key] = mergeDefault(def[key], given[key]);
    }
  }

  return given;
}

function average(arr) {
  return arr.reduce((a, c) => a + c, 0) / arr.length;
}

function format(str, ...replacers) {
  if (typeof str !== 'string') throw new TypeError('Expected a string');
  return str.replace(/{(\d+)}/g, function(match, number) {
    return typeof replacers[number] !== 'undefined' ? replacers[number] : match;
  });
}

function listify(arr, joiner = 'and') {
  const j = ' ' + joiner + ' ';
  if (arr.length <= 2) return arr.join(j);
  return arr.slice(0, -1).join(', ') + j + arr[arr.length - 1];
}

function capFirst(str) {
  return ''.charAt.call(str, 0).toUpperCase() + ''.slice.call(str, 1).toLowerCase();
}

async function asyncForEach(array, callback) {
  for (let i = 0; i < array.length; i ++) {
    await callback(array[i], i, array);
  }
}

function wait(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function logifyDate(date) {
  return '[' + moment(date).format('YYYY-MM-DD HH:mm:ss.SSS [UTC]ZZ') + ']';
}

function savifyDate(date) {
  return logifyDate(date).slice(1, 24).replace(/[^0-9]+/g, '_'); 
}

function makeLogEntry(header, text = '', ...rest) { // jshint ignore:line
  const h = header ? `[${('' + header).toUpperCase()}] ` : '';
  const r = rest.length ? ':\n' + rest.map(e => '  ' + e).join('\n') : '';
  const data = text.trim() + r.trim();
  return logifyDate() + (h.length || data.length ? ': ' : '') + h + data;
}

function logEntryToConsole(...args) {
  console.log(makeLogEntry(...args));
}

function logifyUser(entity) {
  let user = entity.hasOwnProperty('user') ? entity.user : entity;
  return `${user.tag} (${user.id})` + (user.bot ? ' (BOT)' : '');
}

function logifyGuild(guild) {
  return logify(guild) + (guild.available ? '' : ' (Unavailable)');
}

function logifyError(err) {
  var info = err.code && err.path ? err.code + ' ' + err.path : err.code || err.path;
  return `${err.name || 'Error'}: ${err.message}` + (info ? ` (${info})` : '');
}

function logify(obj) {
  return `${obj.name} (${obj.id})`;
}

function stylizeAttachment(attachment) {
  return `${attachment.filename} (${formatBytes(attachment.filesize)}): ${attachment.url}`;
}

function stylizeMetaData(message) {
  let c = message.embeds.length;
  let out = !c ? [] : [`[${c} Embed${c <= 1 ? '' : 's'}]`];
  return [out, ...message.attachments.array().map(stylizeAttachment)];
}

function formatTime(uptime, short = false) {
  if (uptime < 60000) return short ? '<1m' : 'less than a minute';
  const upD = Math.floor(uptime / 8.64e+7);
  const upH = Math.floor(uptime / 3.6e+6) % 24;
  const upM = Math.floor(uptime / 60000) % 60;
  const upDstr = upD + (short ? 'd' : ' day' + (upD === 1 ? '' : 's'));
  const upHstr = upH + (short ? 'h' : ' hour' + (upH === 1 ? '' : 's'));
  const upMstr = upM + (short ? 'm' : ' minute' + (upM === 1 ? '' : 's'));
  return (upD ? upDstr + ', ' : '') + (upD || upH ? upHstr + ' and ' : '') + upMstr;
}

function formatBytes(bytes) {
  return bytes < 1024 ? bytes.toFixed(3) + 'b'
    : bytes < 1048576 ? (bytes / 1024).toFixed(3) + 'kb'
    : bytes < 1073741824 ? (bytes / 1048576).toFixed(3) + 'mb'
    : (bytes / 1073741824).toFixed(3) + 'gb';
}

function escape(str) {
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

function cleanContent(message) {
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

function makeCatcher(logger, msg) {
  return () => logger.log('WARN', msg);
}

function resolveUser(val, client) {
  if (!val || typeof val !== 'string' || !val.trim().length) return null;
  if (client.users.has(val.trim())) return client.users.get(val.trim());
  const match = val.trim().match(/[0-9]+/);
  if (!match) return null;
  return client.users.get(match[0]) || null;
}

function resolveRole(val, guild) {
  if (!val || typeof val !== 'string' || !val.trim().length) return null;
  if (guild.roles.has(val.trim())) return guild.roles.get(val.trim());
  const match = val.trim().match(/[0-9]+/);
  if (!match) return null;
  return guild.roles.get(match[0]) || null;
}

function userOwnsAGuild(user, client) {
  return client.guilds.some((guild) => guild.owner.id === user.id);
}

function fuzzysearch(needle, haystack) {
  var hlen = haystack.length;
  var nlen = needle.length;
  if (nlen > hlen) {
    return false;
  }
  if (nlen === hlen) {
    return needle === haystack;
  }
  outer: for (var i = 0, j = 0; i < nlen; i ++) {
    var nch = needle.charCodeAt(i);
    while (j < hlen) {
      if (haystack.charCodeAt(j ++) === nch) {
        continue outer;
      }
    }
    return false;
  }
  return true;
}

function decancer(str) {
  if (str === void 0 || str === null) return str;
  return Array.from(str.toString().normalize()).map((char) => {
    let p = char.codePointAt(0);
    const alphaNumeric = 
      (p >= 48 && p <= 57) || 
      (p >= 65 && p <= 90) || 
      (p >= 97 && p <= 122);
    if (alphaNumeric) return char.toLowerCase();
    if (charmap.hasOwnProperty(p)) return charmap[p];
    return '';
  }).join('');
}

function getLifetime() {
  process.send({
    head: 'info.lifetime',
    type: 'request'
  });

  return new Promise(function (resolve) {
    process.on('message', function handler(message) {
      if (message.head === 'info.lifetime' && message.type === 'response') {
        resolve(message.data);
      }
    });
  });
}


module.exports = {
  shuffle,
  mergeDefault,
  average,

  format,
  listify,
  capFirst,

  asyncForEach,
  wait,

  logifyDate,
  savifyDate,
  makeLogEntry,
  logEntryToConsole,
  logifyUser,
  logifyGuild,
  logifyError,
  logify,
  stylizeAttachment,
  stylizeMetaData,

  formatTime,
  formatBytes,
  escape,
  cleanContent,

  makeCatcher,

  resolveUser,
  resolveRole,

  userOwnsAGuild,
  fuzzysearch,
  decancer,

  getLifetime
};
