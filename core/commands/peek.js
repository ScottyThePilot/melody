'use strict';
const { RichEmbed } = require('discord.js');
const Command = require('../modules/Command.js');
const { msgFailCatcher } = require('../modules/Logger.js');
const config = require('../config.json');

const infoGroups = [
  { name: 'help', description: 'Shows this menu' },
  { name: 'managers', description: 'Gets info about GuildManagers' },
  { name: 'controller', description: 'Gets general info from the bot controller module' },
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
    usage: `${config.prefix}peek ['help'|'managers'|'controller'|'blacklist'|'file'|'lifetime']`,
    example: `${config.prefix}peek`
  },
  run: async function (bundle) {
    const { controller, message, client, args } = bundle;
    
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
        
      } else if (infoGroup === 'controller') {
        
      } else if (infoGroup === 'blacklist') {
        const blacklist = await controller.blacklist.get('*');
        const joined = blacklist.join('\n').length > 1500 ? blacklist.join('\n').slice(0, 1500) + '...' : blacklist.join('\n');
        const list = '\`\`\`\n' + (blacklist.length === 0 ? 'There are no users on the blacklist.' : joined) + '\n\`\`\`';
        await message.channel.send(`Here is a list of blacklisted IDs:\n${list}`).catch(msgFailCatcher);
      } else if (infoGroup === 'file') {
        
      } else if (infoGroup === 'lifetime') {
        const lifetime = await controller.getLifetime();
        let lifeD = Math.floor(lifetime / 8.64e+7);     lifeD += (lifeD === 1 ? ' day' : ' days');
        let lifeH = Math.floor(lifetime / 3.6e+6) % 24; lifeH += (lifeH === 1 ? ' hour' : ' hours');
        let lifeM = Math.floor(lifetime / 60000) % 60;  lifeM += (lifeM === 1 ? ' minute' : ' minutes');
        await message.channel.send(`The bot manager has been alive for ${lifeD}, ${lifeH}, and ${lifeM}.`).catch(msgFailCatcher);
      } else {
        await message.channel.send('I can\'t find that info group.').catch(msgFailCatcher);
      }
    }
  }
});
