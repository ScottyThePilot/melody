'use strict';
const Command = require('../modules/Command.js');
const config = require('../config.json');
const util = require('../modules/util/util.js');

const modeMap = {
  [undefined]: 0,
  ['']: 0,
  ['add']: 0,
  ['remove']: 1
};

module.exports = new Command({
  name: 'blacklist',
  level: 3,
  plugin: 'owner',
  help: {
    short: 'Blacklists a user.',
    long: 'Puts a user on the bot\'s blacklist, making the bot ignore them. If the second argument is omitted, it defaults to \`\'add\'\`',
    usage: `${config.prefix}blacklist <@mention|user id> [\'add\'|\'remove\']`,
    example: `${config.prefix}blacklist @Scotty#4263 add`
  },
  aliases: ['unblacklist'],
  run: async function ({ melody, message, args, command }) {
    const msgFailCatcher = util.makeCatcher(melody.logger, 'Unable to send message');

    const user = util.resolveUser(args[0], melody.client);

    if (!user) {
      await message.channel.send('Please specify a valid user to blacklist or unblacklist.').catch(msgFailCatcher);
    } else {
      const trusted = [config.ownerID, ...config.trustedUsers].includes(user.id);
      const mode = command === 'unblacklist' ? 1 : modeMap[args[1]] || 0;
      if (mode === 0) {
        if (trusted) {
          await message.channel.send('I cannot blacklist a trusted user!').catch(msgFailCatcher);
        } else {
          const result = await melody.blacklist.add(user);
          if (result) {
            melody.log('INFO', `User ${util.logifyUser(user)} added to the blacklist`);
            await message.channel.send(`Added \`${util.logifyUser(user)}\` to the blacklist.`).catch(msgFailCatcher);
          } else {
            await message.channel.send(`User ${util.logifyUser(user)} is already on the blacklist.`).catch(msgFailCatcher);
          }
        }
      } else {
        const result = await melody.blacklist.remove(user);
        if (result) {
          melody.log('INFO', `User ${util.logifyUser(user)} removed from the blacklist`);
          await message.channel.send(`Removed \`${util.logifyUser(user)}\` from the blacklist.`).catch(msgFailCatcher);
        } else {
          await message.channel.send(`User \`${util.logifyUser(user)}\` is already on the blacklist.`).catch(msgFailCatcher);
        }
      }
    }
  }
});
