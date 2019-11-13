'use strict';
const Command = require('../modules/Command.js');
const config = require('../config.json');
const util = require('../modules/util/util.js');

const challengeMessage = 'has challenged you to a game of connect four. Click the appropriate reaction below to accept or dismiss.';

module.exports = new Command({
  name: 'connectfour',
  level: 0,
  plugin: 'fun',
  help: {
    short: 'Lets you play connect four.',
    long: '',
    usage: `${config.prefix}`,
    example: `${config.prefix}`
  },
  aliases: ['cf'],
  run: async function ({ melody, message, args }) {
    const msgFailCatcher = util.makeCatcher(melody.logger, 'Unable to send message');

    const arg0 = args[0] ? args[0].toLowerCase() : undefined;
    
    if (!arg0) {
      await message.channel.send('That is not a valid subcommand.').catch(msgFailCatcher);
    } else if ('challenge'.startsWith(arg0)) {
      const { user, here } = where(args[0], melody.client, message);
      if (!user) {
        await message.channel.send('I cannot find that user.').catch(msgFailCatcher);
      } else {
        const msg = here
          ? message.channel.send(`<@${user.id}>, <@${message.author.id}> ${challengeMessage}`)
          : user.send(`${message.author.tag} ${challengeMessage}`);
        const result = await msg.then(() => true).catch(msgFailCatcher);
      }
    } else if ('move'.startsWith(arg0)) {
      
    } else if ('forfeit'.startsWith(arg0)) {
      
    } else if ('pending'.startsWith(arg0)) {

    }
  }
});

function where(userResolvable, client, message) {
  const user = util.resolveUser(userResolvable, client);
  const here = user ? message.guild && message.guild.users.has(user.id) : false;
  return { user, here };
}

/* sending a challenge
if in dm or player not in server
  send challenge to other player's dm
else 
*/
