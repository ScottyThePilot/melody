'use strict';
const Command = require('../modules/Command.js');
const config = require('../config.json');
const util = require('../modules/util/util.js');
const { Attachment } = require('discord.js');

module.exports = new Command({
  name: 'dump',
  plugin: 'core',
  help: {
    short: 'Gets message logs.',
    long: 'Sends you a file containing server logs for a specified guild you own.\nNOTE: Message logging cannot determine what user deleted a message.',
    usage: `${config.prefix}dump [serverID]`,
    example: `${config.prefix}dump 750539831957333711`
  },
  run: async function ({ melody, message, args }) {
    const msgFailCatcher = util.makeCatcher(melody.logger, 'Unable to send message');
    const ownedGuilds = message.author.id === config.ownerID ? message.client.guilds
      : message.client.guilds.filter((guild) => guild.ownerID === message.author.id);

    if (message.author.id === config.ownerID && (args[0] || '').startsWith('activity')) {
      await message.author.send(`Core Bot Logs:`, {
        file: new Attachment(melody.logger.path)
      }).catch((error) => {
        console.log(error);
        message.author.send(`Unable to attach file. \`Error: ${error.code}\``).catch(msgFailCatcher);
      });
    } else if (ownedGuilds.size < 1) {
      await message.author.send('There are no logs to dump as you do not own any servers.');
    } else if (ownedGuilds.size === 1) {
      const guild = ownedGuilds.first();
      const manager = melody.guildManagers.get(guild.id);

      await message.author.send(`Message logs for ${guild.name}:`, {
        file: new Attachment(manager.logger.path, `guild${guild.id}.log`)
      }).catch((error) => {
        console.log(error);
        message.author.send(`Unable to attach file. \`Error: ${error.code}\``).catch(msgFailCatcher);
      });
      
      message.author.dmChannel.stopTyping();
    } else if (!args[0]) {
      await message.author.send('Because you own more than one guild, you must specify a server ID to dump logs for.').catch(msgFailCatcher);
    } else {
      if (ownedGuilds.has(args[0])) {
        const guild = ownedGuilds.get(args[0]);
        const manager = melody.guildManagers.get(guild.id);

        await message.author.send(`Message logs for ${guild.name}:`, {
          file: new Attachment(manager.logger.path, `guild${guild.id}.log`)
        }).catch((error) => {
          console.log(error);
          message.author.send(`Unable to attach file. \`Error: ${error.code}\``).catch(msgFailCatcher);
        });
      } else {
        await message.author.send('I can\'t find that guild.').catch(msgFailCatcher);
      }
    }
  }
});
