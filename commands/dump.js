'use strict';
const { Attachment } = require('discord.js');
const Command = require('../modules/Command.js');
const { msgFailCatcher } = require('../modules/Logger.js');
const config = require('../config.json');
const fs = require('fs');

module.exports = new Command({
  name: 'dump',
  level: 1,
  plugin: 'core',
  help: {
    short: 'Gets message logs.',
    long: 'Sends you a file containing server logs for a specified guild you own.\nNOTE: Message logging cannot determine what user deleted a message.',
    usage: `${config.prefix}dump [serverID]`,
    example: `${config.prefix}dump 750539831957333711`
  },
  run: async function (bundle) {
    const { message, args } = bundle;
    const ownedGuilds = message.author.id === config.ownerID ? message.client.guilds
      : message.client.guilds.filter((guild) => guild.ownerID === message.author.id);

    if (message.author.id === config.ownerID && (args[0] || '').startsWith('activity')) {
      await message.author.send(`Core Bot Logs:`, {
        file: new Attachment('./activity.log', 'activity.log')
      }).catch((reason) => {
        console.log(reason);
        message.author.send('Unable to attach file.').catch(msgFailCatcher);
      });
    } else if (ownedGuilds.size < 1) {
      await message.author.send('There are no logs to dump as you do not own any servers.');
    } else if (ownedGuilds.size === 1) {
      const guild = ownedGuilds.first();
      if (fs.existsSync(`./data/${guild.id}/latest.log`)) {
        await message.author.send(`Message logs for ${guild.name}:`, {
          file: new Attachment(`./data/${guild.id}/latest.log`, `guild${guild.id}.log`)
        }).catch((reason) => {
          console.log(reason);
          message.author.send('Unable to attach file.').catch(msgFailCatcher);
        });
      } else {
        await message.author.send(`${guild.name} has no message logs.`).catch(msgFailCatcher);
      }
    } else if (!args[0]) {
      await message.author.send('Because you own more than one guild, you must specify a server ID to dump logs for.').catch(msgFailCatcher);
    } else {
      if (ownedGuilds.has(args[0])) {
        const guild = ownedGuilds.get(args[0]);
        if (fs.existsSync(`./data/${guild.id}/latest.log`)) {
          await message.author.send(`Message logs for ${guild.name}:`, {
            file: new Attachment(`./data/${guild.id}/latest.log`, `guild${guild.id}.log`)
          }).catch((reason) => {
            console.log(reason);
            message.author.send('Unable to attach file.').catch(msgFailCatcher);
          });
        } else {
          await message.author.send(`${guild.name} has no message logs.`).catch(msgFailCatcher);
        }
      } else {
        await message.author.send('').catch(msgFailCatcher);
      }
    }
  }
});