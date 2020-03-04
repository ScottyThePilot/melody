'use strict';
const moment = require('moment');

/**
 * @param {Date} date 
 */
function logifyDate(date) {
  return '[' + moment(date).format('YYYY-MM-DD HH:mm:ss.SSS [UTC]ZZ') + ']';
}

/**
 * @param {Date} date 
 */
function savifyDate(date) {
  return logifyDate(date).slice(1, 24).replace(/[^0-9]+/g, '_'); 
}

/**
 * @param {string} header
 * @param {string} [text]
 * @param {...string} [rest]
 * @returns {string}
 */
function makeLogEntry(header, text = '', ...rest) { // jshint ignore:line
  const h = header ? `[${('' + header).toUpperCase()}] ` : '';
  const r = rest.length ? ':\n' + rest.map(e => '  ' + e).join('\n') : '';
  const data = text.trim() + r.trim();
  return logifyDate() + (h.length || data.length ? ': ' : '') + h + data;
}

/**
 * @param {User|GuildMember} entity
 * @returns {string}
 */
function logifyUser(entity) {
  let user = entity.hasOwnProperty('user') ? entity.user : entity;
  return `${user.tag} (${user.id})` + (user.bot ? ' (BOT)' : '');
}

/**
 * @param {Guild} guild
 * @returns {string}
 */
function logifyGuild(guild) {
  return logify(guild) + (guild.available ? '' : ' (Unavailable)');
}

/**
 * @param {Error} err
 * @returns {string}
 */
function logifyError(err) {
  let info = err.code && err.path ? err.code + ' ' + err.path : err.code || err.path;
  return `${err.name || 'Error'}: ${err.message}` + (info ? ` (${info})` : '');
}

/**
 * @param {{ name: string, id: string }} obj
 * @returns {string}
 */
function logify(obj) {
  return `${obj.name} (${obj.id})`;
}

module.exports = {
  makeLogEntry,

  logify,
  logifyDate,
  logifyUser,
  logifyGuild,
  logifyError,
  savifyDate
};

/**
 * @typedef {import('discord.js').Guild} Guild
 * @typedef {import('discord.js').GuildMember} GuildMember
 * @typedef {import('discord.js').User} User
 */
