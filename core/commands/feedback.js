'use strict';
const Command = require('../structures/Command.js');
const config = require('../config.json');
const util = require('../modules/util.js');
const { RichEmbed } = require('discord.js');

module.exports = new Command({
  name: 'feedback',
  plugin: 'core',
  help: {
    short: 'Send some feedback.',
    long: 'Sends a message to the bot owner. Feel free to leave any suggestions, questions, comments, or criticism you have.',
    usage: `${config.prefix}feedback [message]`,
    example: `${config.prefix}feedback I like this bot!`
  },
  run: async function ({ melody, message, command }) {
    const msgFailCatcher = util.makeCatcher(melody.logger, 'Unable to send message');

    const msg = util.cleanContent(message).trim().slice(config.prefix.length + command.length).trim();
    
    if (msg.length > 16) {
      const embed = new RichEmbed();

      embed.setTitle('*Provides some feedback...*');
      embed.setDescription(msg);
      embed.setAuthor(message.author.tag, message.author.displayAvatarURL);
      embed.setColor([114, 137, 218]);

      melody.client.users.get(config.ownerID).send(embed).then(() => {
        message.channel.send('Thank you for your feedback!').catch(msgFailCatcher);
      }).catch(msgFailCatcher);
    } else {
      message.channel.send('Invalid Feedback!').catch(msgFailCatcher);
    }
  }
});
