'use strict';
const { RichEmbed } = require('discord.js');
const Command = require('../modules/Command.js');
const { msgFailCatcher, cleanContent } = require('../modules/Logger.js');
const config = require('../config.json');

module.exports = new Command({
  name: 'feedback',
  plugin: 'core',
  help: {
    short: 'Send some feedback.',
    long: 'Sends a message to the bot owner. Feel free to leave any suggestions, questions, comments, or criticism you have.',
    usage: `${config.prefix}feedback [message]`,
    example: `${config.prefix}feedback I like this bot!`
  },
  run: async function (bundle) {
    const { message, client, command } = bundle;

    const msg = cleanContent(message).trim().slice(config.prefix.length + command.length).trim();
    
    if (msg.length > 16) {
      const embed = new RichEmbed();

      embed.setTitle('*Provides some feedback...*');
      embed.setDescription(msg);
      embed.setAuthor(message.author.tag, message.author.avatarURL);
      embed.setColor([114, 137, 218]);

      client.users.get(config.ownerID).send(embed).then(function () {
        message.channel.send('Thank you for your feedback!').catch(msgFailCatcher);
      }).catch(msgFailCatcher);
    } else {
      message.channel.send('Invalid Feedback!').catch(msgFailCatcher);
    }
  }
});
