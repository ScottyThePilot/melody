'use strict';
const Command = require('../structures/Command.js');
const config = require('../config.json');
const util = require('../modules/util.js');

const { BOOLEAN_KEYWORDS: boolWords } = require('../modules/constants.js');

const aliases = [
  'cleverbotzones',
  'cleverzone',
  'cleverzones',
  'cbzone',
  'cbzones'
];

module.exports = new Command({
  name: 'cleverbotzone',
  plugin: 'fun',
  level: 1,
  disabled: true,
  help: {
    short: 'Turn a channel into a CleverBot zone.',
    long: 'Turns a channel into a CleverBot zone. Melody will send a CleverBot response to all messages sent in CleverBot zones.',
    usage: `${config.prefix}cleverbotzone <'enable'|'disable'>`,
    example: `${config.prefix}cleverbotzone enable`
  },
  aliases,
  inDM: false,
  run: async function ({ melody, message, manager, args }) {
    const msgFailCatcher = util.makeCatcher(melody.logger, 'Unable to send message');

    const state = manager.configdb
      .getSync('cleverBotZones')
      .includes(message.channel.id);

    if (!args[0]) {
      const msg = state
        ? `This channel is not a CleverBot zone. Use \`${config.prefix}cleverbotzone disable\` to disable this channel as a CleverBot zone.`
        : `This channel is a CleverBot zone. Use \`${config.prefix}cleverbotzone enable\` to enable this channel as a CleverBot zone.`;
      await message.channel.send(msg).catch(msgFailCatcher);
    } else {
      const word = args[1] ? args[1].toLowerCase() : undefined;

      if (!word || !boolWords.hasOwnProperty(word)) {
        await message.channel.send().catch(msgFailCatcher);
      } else {
        const desired = boolWords[word];

        if (desired === state) {
          await message.channel.send(`CleverBot zone is already ${state ? 'enabled' : 'disabled'} in this channel.`).catch(msgFailCatcher);
        } else if (desired) {
          // Enable CleverBot zone
          await manager.configdb.edit((data) => {
            data.cleverBotZones.push(message.channel.id);
          });
          await message.channel.send('This channel is now a CleverBot zone.').catch(msgFailCatcher);
        } else {
          // Disable CleverBot zone
          await manager.configdb.edit((data) => {
            data.cleverBotZones.push(message.channel.id);
          });
          await message.channel.send('This channel is no longer a CleverBot zone.').catch(msgFailCatcher);
        }
      }
    }
  }
});
