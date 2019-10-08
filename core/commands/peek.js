'use strict';
const Command = require('../modules/Command.js');
const config = require('../config.json');
const util = require('../modules/util/util.js');
const { RichEmbed } = require('discord.js');

const infoGroups = [
  { name: 'help', description: 'Shows this menu' },
  { name: 'managers', description: 'Gets info about GuildManagers' },
  { name: 'blacklist', description: 'Lists IDs in the blacklist' },
  { name: 'file', description: 'Gets info about files and data' },
  { name: 'lifetime', description: 'How long the bot manager has been alive' }
];

module.exports = new Command({
  name: 'peek',
  plugin: 'owner',
  level: 3,
  disabled: true,
  help: {
    short: 'Retrieves stored info.',
    long: 'Allows memory, database data, and other information to be easily accessed.',
    usage: `${config.prefix}peek ['help'|'managers'|'blacklist'|'file'|'lifetime']`,
    example: `${config.prefix}peek`
  },
  run: async function ({ melody, message, args }) {
    const msgFailCatcher = util.makeCatcher(melody.logger, 'Unable to send message');
    
    if (!args[0]) {
      await message.channel.send(`Please provide an info group to peek. You can list info groups with \`${config.prefix}peek help\`.`).catch(msgFailCatcher);
    } else {
      const infoGroup = args[0].toLowerCase();
      if (infoGroup === 'help') {
        const body = infoGroups.map((g) => `\`${config.prefix}peek ${g.name}\`: *${g.description}*`).join('\n');

        const embed = new RichEmbed();

        embed.setTitle('Peek Command Help');
        embed.setDescription(`Below is a list of info groups usable in the \`${config.prefix}peek\` command, and a description of what they do.`);
        embed.setColor([114, 137, 218]);
        embed.addField('Info Group List', body);
        embed.setFooter(`Melody v${config.version[1]} ${config.version[0]}`);

        await message.channel.send(embed).catch(msgFailCatcher);
      } else if (infoGroup === 'managers') {
        
      } else if (infoGroup === 'blacklist') {
        const blacklist = await melody.blacklist.db.get('*');
        const joined = blacklist.join('\n').length > 1500 ? blacklist.join('\n').slice(0, 1500) + '...' : blacklist.join('\n');
        const list = '\`\`\`\n' + (blacklist.length === 0 ? 'There are no users on the blacklist.' : joined) + '\n\`\`\`';
        await message.channel.send(`Here is a list of blacklisted IDs:\n${list}`).catch(msgFailCatcher);
      } else if (infoGroup === 'file') {
        
      } else if (infoGroup === 'lifetime') {
        const lifetime = util.formatTime(); // @TODO
        await message.channel.send(`The bot manager has been alive for ${lifetime}.`).catch(msgFailCatcher);
      } else {
        await message.channel.send('I can\'t find that info group.').catch(msgFailCatcher);
      }
    }
  }
});
