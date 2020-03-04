'use strict';
const { mergeDefault } = require('../modules/utils/object.js');

class Command {
  /**
   * @param {CommandOptions} opts
   */
  constructor(opts) {
    const options = mergeDefault(Command.defaultOptions, opts);

    if (typeof options.run !== 'function')
      throw new Error('options.run is not a function');

    /** @type {string} */
    this.name = options.name;

    /** @type {CommandHelp} */
    this.help = options.help;
    
    /** @type {string[]} */
    this.aliases = options.aliases;
    
    /** @type {object} */
    this.info = options.info;
    
    /** @type {'dm'|'guild'|'anywhere'} */
    this.where = options.where;
    
    /** @type {boolean} */
    this.disabled = options.disabled;

    /** @type {number|string} */
    this.level = options.level;
    
    /** @type {boolean} */
    this.hidden = options.hidden;
    
    /** @type {async (data: object) => any} */
    this.run = options.run;
  }

  get levelString() {
    switch (this.level) {
      case 0: return 'Everyone';
      case 1: return 'Server administrators';
      case 2: return 'Server owners';
      case 3: return 'Trusted Users';
      case 10: return 'Bot owner';
      default: return this.level;
    }
  }

  /**
   * Try to execute a command with the given data object. The command will not execute
   * if it is disabled, or if its `where` property does not match up with whether it is
   * in DM or a guild.
   * @param {object} data A command data object
   * @returns {Promise} A promise resolving when the command finishes,
   *   or to `null` if the command did not execute.
   */
  async attempt(data) {
    if (this.disabled) return null;

    const inDM = !data.message.guild;

    if (!inDM && this.where === Command.DM) return null;
    if (inDM && this.where === Command.GUILD) return null;
    if (data.level < this.level) return null;

    await this.run.call(this, data);
  }

  /**
   * Check whether a query matches this command's name or one of its aliases.
   * @param {string} query The command name to check
   * @returns {boolean} Whether it matches or not
   */
  is(query) {
    if (this.name === query) return true;
    for (let alias of this.aliases)
      if (alias === query) return true;
    return false;
  }

  /**
   * Search a list of commands for one with the given name.
   * @param {Collection<Command>} commands A set of commands to search through
   * @param {string} query The name or alias of a command to find
   * @returns {Command|null} The command object if one was found, or `null` if one wasn't
   */
  static find(commands, query) {
    for (let command of commands)
      if (command.is(query)) return command;
    return null;
  }
}

Command.DM = 'dm';
Command.GUILD = 'guild';
Command.ANYWHERE = 'anywhere';

Command.defaultOptions = {
  name: 'default',
  help: {
    short: 'Invalid',
    long: 'Invalid',
    usage: 'Invalid',
    example: 'Invalid'
  },
  aliases: [],
  info: {},
  where: 'anywhere',
  level: 0,
  disabled: false,
  hidden: false,
  run: null
};

module.exports = Command;

/**
 * @typedef CommandOptions
 * @property {string} name
 * @property {CommandHelp} help
 * @property {string[]} aliases
 * @property {object} info
 * @property {'dm'|'guild'|'anywhere'} where
 * @property {number|string} level
 * @property {boolean} disabled
 * @property {boolean} hidden
 * @property {async (data: object) => any} run
 */

/**
 * @typedef CommandHelp
 * @property {string} short
 * @property {string} long
 * @property {string} usage
 * @property {string} example
 */
