'use strict';
const Command = require('../structures/Command.js');
const config = require('../config.json');
const util = require('../modules/util.js');
const { RichEmbed } = require('discord.js');


const contents = require('../static/changeloglatest.json');
const embed = new RichEmbed();

const extra = 'Read the full changelog [here](https://github.com/ScottyThePilot/melody_v3/blob/master/changelog.md).';
const description = (contents.description ? contents.description + '\n' : '') + extra;

embed.setTitle(contents.title);
embed.setDescription(description);
embed.setColor([114, 137, 218]);
contents.fields.forEach((f) => embed.addField(f.name, f.value));


module.exports = new Command({
  name: 'changelog',
  plugin: 'core',
  help: {
    short: 'Gets the latest changes.',
    long: 'Grabs the latest changelog entry, listing recent bot functionality updates.',
    usage: `${config.prefix}changelog`,
    example: `${config.prefix}changelog`
  },
  run: async function ({ melody, message }) {
    const msgFailCatcher = util.makeCatcher(melody.logger, 'Unable to send message');

    message.channel.send(embed).catch(msgFailCatcher);
  }
});
