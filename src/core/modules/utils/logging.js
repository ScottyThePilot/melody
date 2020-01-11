'use strict';
const moment = require('moment');

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
  let info = err.code && err.path ? err.code + ' ' + err.path : err.code || err.path;
  return `${err.name || 'Error'}: ${err.message}` + (info ? ` (${info})` : '');
}

function logify(obj) {
  return `${obj.name} (${obj.id})`;
}

module.exports = {
  makeLogEntry,

  logEntryToConsole,

  logify,
  logifyDate,
  logifyUser,
  logifyGuild,
  logifyError,
  savifyDate
};
