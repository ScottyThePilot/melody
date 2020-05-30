'use strict';
import Discord from 'discord.js';
import moment from 'moment';
import { inspect } from 'util';

export default class Util {
  constructor() {
    throw new Error('The Util class cannot be constructed');
  }

  static async suppressCode(promise, code) {
    try {
      return await promise;
    } catch ({ code: c }) {
      return c === code ? undefined : await promise;
    }
  }

  /**
   * @param {number} ms
   * @returns {Promise}
   */
  static wait(ms) {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  /**
   * @param {import("events").EventEmitter} emitter
   * @param {string | symbol} event
   * @returns {Promise<any[]>}
   */
  static onceEvent(emitter, event) {
    return new Promise((resolve) => emitter.once(event, (...args) => resolve(args)));
  }

  /**
   * @param {Date} [date]
   * @returns {string}
   */
  static logifyDate(date) {
    return '[' + moment(date).format('YYYY-MM-DD HH:mm:ss.SSS [UTC]ZZ') + ']';
  }

  /**
   * @param {Date} [date]
   * @returns {string}
   */
  static savifyDate(date) {
    return Util.logifyDate(date).slice(1, 24).replace(/[^0-9]+/g, '_'); 
  }

  /**
   * @param {string} header
   * @param {string} [text='']
   * @param {...string} [rest]
   * @returns {string}
   */
  static makeLogEntry(header, text, ...rest) {
    const h = header ? `[${('' + header).toUpperCase()}] ` : '';
    const r = rest.length ? ':\n' + rest.map(e => '  ' + e).join('\n') : '';
    const data = text.trim() + r.trim();
    return Util.logifyDate() + (h.length || data.length ? ': ' : '') + h + data;
  }

  /**
   * @param {Discord.GuildMember | Discord.User} entity
   * @returns {string}
   */
  static logifyUser(entity) {
    const user = entity instanceof Discord.GuildMember ? entity.user : entity;
    return `${user.tag} (${user.id})` + (user.bot ? ' (BOT)' : '');
  }

  /**
   * @param {Discord.Guild}
   * @returns {string}
   */
  static logifyGuild(guild) {
    return Util.logify(guild) + (guild.available ? '' : ' (Unavailable)');
  }

  /**
   * @param {Error} err
   * @returns {string}
   */
  static logifyError(err) {
    const info = err instanceof Discord.HTTPError || err instanceof Discord.DiscordAPIError ? `(${err.code} ${err.path})` : '';
    const message = Util.joinAny(err.message, info) || inspect({ ...err }, true);
    return `${err.name || 'Unknown Error'}: ${message}`;
  }

  /**
   * @param {{ name: string, id: string }} obj
   * @returns {string}
   */
  static logify({ name, id }) {
    return `${name} (${id})`;
  }

  /**
   * @param {string} str
   * @returns {string}
   */
  static capFirst(str) {
    if (str.length === 0) return '';
    const a = String.fromCodePoint(str.codePointAt(0)).toUpperCase();
    return a + str.slice(a.length).toLowerCase();
  }

  /**
   * @param {...any} strings
   * @returns {string}
   */
  static joinAny(...strings) {
    let out = '';
    for (const str of strings) {
      if (typeof str !== 'string' || !str.trim()) continue;
      if (out) out += ' ';
      out += str.trim();
    }
    return out;
  }

  /**
   * @param {object} def
   * @param {object} given
   * @returns {object}
   */
  static mergeDefault(def, given) {
    if (!given) return def;
    for (const key in def) {
      if (!{}.hasOwnProperty.call(given, key)) {
        given[key] = def[key];
      } else if (given[key] === Object(given[key])) {
        given[key] = Util.mergeDefault(def[key], given[key]);
      }
    }

    return given;
  }

  /**
   * @param {object} obj
   * @param {string | string[]} path
   * @returns {any}
   */
  static get(obj, path) {
    const a = validate(obj, path);
    if (a === null) return obj;

    for (let key of a) {
      if (key in obj) {
        obj = obj[key];
      } else {
        return;
      }
    }

    return obj;
  }

  /**
   * @param {object} obj
   * @param {string | string[]} path
   * @param {any} value
   */
  static set(obj, path, value) {
    const a = validate(obj, path);
    if (a === null) return;

    while (a.length > 1) {
      let key = a.shift();
      let v = obj[key];
      obj = obj[key] =
        typeof v === 'object' && v !== null
          ? v : isNaN(a[0]) ? {} : [];
    }

    obj[a[0]] = value;
  }

  /**
   * @param {object} obj
   * @param {string | string[]} path
   * @returns {boolean}
   */
  static has(obj, path) {
    const a = validate(obj, path);
    if (a === null) return true;

    for (let key of a) {
      if (key in obj) {
        obj = obj[key];
      } else {
        return false;
      }
    }

    return true;
  }
}

/**
 * @param {object} obj
 * @param {string | string[]} path
 * @returns {string[] | null}
 */
function validate(obj, path) {
  if (obj === null || obj === undefined) throw new Error('Invalid Object: ' + obj);
  if (path === undefined || !(Array.isArray(path) ? path : '' + path).length)
    return null;
  const a = Array.isArray(path)
    ? path
    : path
      .replace(/\[(\w+)\]/g, '.$1')
      .replace(/^\./, '')
      .split('.');
  if (a.some(key => !(/^(?:[0-9]|[a-zA-Z_$][a-zA-Z_$0-9\-]*)$/).test(key))) // jshint ignore: line
    throw new Error('Invalid Path');
  return a;
}

