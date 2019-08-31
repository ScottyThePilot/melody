'use strict';
const { RichEmbed } = require('discord.js');
const Command = require('../modules/Command.js');
const { msgFailCatcher } = require('../modules/Logger.js');
const config = require('../config.json');
const { readFile } = require('fs');

const contents = new Promise((resolve, reject) => {
  readFile('./core/changeloglatest.txt', (err, data) => {
    if (err) {
      reject(err);
    } else {
      resolve(data
        .toString()
        .replace(/\r/g, '')
        .split(/\n{2,}/)
        .map((e) => e.trim()));
    }
  });
});

module.exports = new Command({
  name: 'changelog',
  plugin: 'core',
  help: {
    short: 'Gets the latest changes.',
    long: 'Grabs the latest changelog entry, listing recent bot functionality updates.',
    usage: `${config.prefix}feedback [message]`,
    example: `${config.prefix}feedback I like this bot!`
  },
  run: async function (bundle) {
    const { message } = bundle;

    const [version, entry] = await contents;

    const embed = new RichEmbed();

    embed.setTitle(version);
    embed.setDescription(entry);
    embed.setColor([114, 137, 218]);

    message.channel.send(embed).catch(msgFailCatcher);
  }
});
