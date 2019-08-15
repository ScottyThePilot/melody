'use strict';
const Command = require('../modules/Command.js');
const { msgFailCatcher } = require('../modules/Logger.js');
const config = require('../config.json');

module.exports = new Command({
  name: 'help',
  help: {
    short: 'Gets command help.',
    long: 'Gets the current latency.',
    usage: `${config.prefix}help [command]`,
    example: `${config.prefix}help ping`
  },
  aliases: ['halp', 'h'],
  run: async function run(bundle) {
    const { message, client } = bundle;
    const args = bundle.args.map((arg) => arg.toLowerCase());
    
    if (args[0]) {
      
    } else {

    }
  }
});