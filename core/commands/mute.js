'use strict'; /*
const Command = require('../modules/Command.js');
const config = require('../config.json');
const util = require('../modules/util/util.js');

module.exports = new Command({
  name: 'mute',
  level: 3,
  plugin: 'moderation',
  help: {
    short: 'Blacklists a user.',
    long: 'Puts a user on the bot\'s blacklist, making the bot ignore them. If the second argument is omitted, it defaults to \`\'add\'\`',
    usage: `${config.prefix}blacklist <@mention|user id> [\'add\'|\'remove\']`,
    example: `${config.prefix}blacklist @Scotty#4263 add`
  },
  aliases: ['unblacklist'],
  run: async function ({ melody, message, args }) {
    
  }
});


function findRoleByName(roles, name, count = 5) {
  let clean = ('' + name).trim();
  let targetName = util.decancer(clean);
  return [...roles.values()].filter((role) => {
    let roleName = util.decancer(role.name);
    return clean === role.name.trim() ||
      util.fuzzysearch(targetName, roleName);
  }).slice(0, count);
}*/
