'use strict';
const Command = require('../structures/Command.js');
const config = require('../config.json');
const util = require('../modules/util.js');
const leven = require('leven');

module.exports = new Command({
  name: 'mute',
  level: 0,
  plugin: 'moderation',
  help: {
    short: 'Mutes a user.',
    long: `Gives a user the current server\'s muted role. Make sure to set up your server\'s muted role with \`${config.prefix}configure set mutedRole <role id>\`.`,
    usage: `${config.prefix}mute <@mention|user id|user tag> [duration] | ${config.prefix}mute <username>`,
    example: `${config.prefix}mute @User#0000 1 day`
  },
  inDM: false,
  aliases: ['unmute'],
  run: async function ({ melody, message, args, argsText, command }) {
    const msgFailCatcher = util.makeCatcher(melody.logger, 'Unable to send message');

    if (!args[0]) {
      
    } else {
      const resolved = util.resolveUserAdvanced(message.guild, argsText);
      
      if (resolved && message.guild.members.has(resolved.user.id)) {
        const { match, user } = resolved;
        const rest = argsText.slice(match.length).trim();

        const future = rest.length > 0 ? util.parseFuture(rest) : null;



      } else {
        // The top 9 ranking matches to the given query
        const candidates = getCandidates(message.guild.members, argsText).slice(0, 9);
        
      }
    }
  }
});

function getCandidates(members, resolvable) {
  return members
    .map((member) => {
      const { username } = member.user;
      const rating = Math.min(
        leven(username, resolvable),
        leven(util.decancer(username), resolvable)
      );
      return [member, rating];
    })
    .sort((m1, m2) => {
      return m1[1] - m2[1];
    });
}
