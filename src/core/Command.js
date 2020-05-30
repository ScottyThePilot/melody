'use strict';

export default class Command {
  /**
   * @param {Options} options
   */
  constructor({
    name, help, exec,
    aliases = [],
    info = {},
    where = 'anywhere',
    disabled = false,
    level = 0,
    hidden = false
  }) {
    /** @type {string} */
    this.name = name;
    /** @type {{ short: string, long: string, usage: string, example: string }} */
    this.help = help;
    /** @type {string[]} */
    this.aliases = aliases;
    /** @type {object} */
    this.info = info;
    /** @type {Where} */
    this.where = where;
    /** @type {boolean} */
    this.disabled = disabled;
    /** @type {number} */
    this.level = level;
    /** @type {boolean} */
    this.hidden = hidden;
    /** @type {Executor} */
    this.exec = exec;
  }

  /** @type {string} */
  get group() {
    if (typeof this.level === 'string')
      return this.level;
    switch (this.level) {
      case 0: return 'Everyone';
      case 1: return 'Server Administrators';
      case 2: return 'Server Owners';
      case 3: return 'Trusted Users';
      case 10: return 'Bot owner';
      default: return 'Unknown';
    }
  }

  /**
   * @param {Data} data
   * @returns {Promise<string>}
   */
  async attempt(data) {
    if (this.disabled) return 'disabled';
    if (!this.here(data.location)) return 'not_here';
    if (data.level < this.level) return 'wrong_level';

    

    return 'ok';
  }

  /**
   * @param {Location} location 
   * @returns {boolean}
   */
  here(location) {
    return this.where === 'anywhere' ? true : this.where === location;
  }

  /**
   * @param {string} query
   * @returns {boolean}
   */
  is(query) {
    const q = query.toLowerCase();
    if (this.name === q) return true;
    for (const alias of this.aliases)
      if (alias.toLowerCase() === q) return true;
    return false;
  }
}

/**
 * @typedef {'dm' | 'guild'} Where
 * @typedef {'dm' | 'guild' | 'location'} Location
 */

/**
 * @typedef {(this: Command, data: CommandData) => Promise<void>} ExecutorFunction
 * @typedef {ExecutorFunction | { [key: string]: ExecutorFunction }} Executor
 */

/**
 * @typedef DataBasic
 * @property {import('discord.js').Message} message
 * @property {string} command
 * @property {string} argsText
 * @property {string[]} args
 */

/**
 * @typedef Data
 * @property {import('./Melody')} melody
 * @property {number} level
 * @property {Where} where
 * @property {import('./Manager') | null} manager
 * @property {import('discord.js').Message} message
 * @property {string} command
 * @property {string} argsText
 * @property {string[]} args
 */

/**
 * @typedef Options
 * @property {string} name
 * @property {object} help
 * @property {string} help.short
 * @property {string} help.long
 * @property {string} help.usage
 * @property {string} help.example
 * @property {string[]} [aliases]
 * @property {object} info
 * @property {Where} [where]
 * @property {boolean} [disabled]
 * @property {number} [level]
 * @property {boolean} [hidden]
 * @property {Executor} exec
 */


