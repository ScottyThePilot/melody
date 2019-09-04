'use strict';
const { RichEmbed } = require('discord.js');
const Command = require('../modules/Command.js');
const { msgFailCatcher } = require('../modules/Logger.js');
const config = require('../config.json');

const contents = require('../changeloglatest.json');
const embed = new RichEmbed();

const extra = 'Read the full changelog [here](https://github.com/ScottyThePilot/melody_v3/blob/master/changelog.md)';
const description = (contents.description ? contents.description + '\n' : '') + extra;

embed.setTitle(contents.title);
embed.setDescription(description);
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
    usage: `${config.prefix}changelog`,
    example: `${config.prefix}changelog`
  },
  run: async function (bundle) {
    const { message } = bundle;

    message.channel.send(embed).catch(msgFailCatcher);
  }
});