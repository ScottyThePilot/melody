'use strict';
const { RichEmbed } = require('discord.js');
const Command = require('../modules/Command.js');
const { msgFailCatcher } = require('../modules/Logger.js');
const config = require('../config.json');

const contents = require('../changeloglatest.json');
const embed = new RichEmbed();

embed.setTitle(contents.title);
embed.setDescription(contents.description);
embed.setColor([114, 137, 218]);
[].forEach.call(contents.fields, function (field) {
  embed.addField(field.name, field.value);
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

    message.channel.send(embed).catch(msgFailCatcher);
  }
});
