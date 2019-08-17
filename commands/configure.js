'use strict';
const Command = require('../modules/Command.js');
const { msgFailCatcher } = require('../modules/Logger.js');
const config = require('../config.json');

const configProperties = [
  //['trackInvites', 'bool'],
  //['preserveRoles', 'bool'],
  ['logMessages', 'bool', `If \`logMessages\` is true, the bot will log message edits and deletions. Logs can be retrieved with \`${config.prefix}dump\`, or cleared with \`${config.prefix}flush\`.`],
  //['autoMod', 'bool'],
  //['antiSpam', 'bool'],
  //['mutedRole', 'role']
];

const configBoolMap = {
  'true': true,
  't': true,
  'false': false,
  'f': false,
  'enable': true,
  'enabled': true,
  'e': true,
  'disable': false,
  'disabled': false,
  'd': false,
  'on': true,
  'off': false,
  'yes': true,
  'y': true,
  'no': false,
  'n': false,
  '0': false,
  '1': true
};

module.exports = new Command({
  name: 'configure',
  level: 2,
  plugin: 'core',
  help: {
    short: 'Changes server config settings.',
    long: 'Allows the server owner to modify server configuration settings for the bot. Exclude the \`value\` argument to get the current value of a property or exclude the \`config property\` argument to list all valid properties.',
    usage: `${config.prefix}configure [config property] [value]`,
    example: `${config.prefix}configure logMessages enable`
  },
  aliases: ['config', 'cfg'],
  inDM: false,
  run: async function run(bundle) {
    const { message, manager, args } = bundle;

    if (!args[0]) {
      const propList = configProperties.map((p) => `\`${p[0]}\``).join(', ');

      await message.channel.send(`Valid config properties are:\n${propList}`).catch(msgFailCatcher);
    } else {
      const propName = getConfigProp(args[0]);

      if (!propName) {
        await message.channel.send('That is not a valid config property.').catch(msgFailCatcher);
      } else if (!args[1]) {
        const currentVal = await manager.configdb.get(propName);
        const info = configProperties.find((p) => p[0] === propName)[2];

        await message.channel.send(`The current value of \`${propName}\` is \`${currentVal}\`.\n${info}`).catch(msgFailCatcher);
      } else {
        const type = configProperties.find((p) => p[0] === propName)[1];
        const verified = type === 'bool' ? verifyBool(args[1])
          : type === 'role' ? verifyRole(args[1], message.guild.roles)
          : undefined;

        if (!verified) {
          await message.channel.send('That is not a valid value.').catch(msgFailCatcher);
        } else {
          const value = type === 'bool' ? boolify(args[1])
            : type === 'role' ? args[1]
            : undefined;

          await manager.configdb.set(propName, value);
          manager.log('DATA', `Config property ${propName} updated to ${value}`);
          await message.channel.send(`The value of \`${propName}\` has been updated to \`${value}\``);
        }
      }
    }
  }
});

function getConfigProp(name) {
  const n = name.replace(/\W+/g, '').toLowerCase();
  return configProperties.find((p) => p[0].toLowerCase() === n)[0];
}

function verifyBool(val) {
  return configBoolMap.hasOwnProperty(val.toLowerCase());
}

function boolify(val) {
  return configBoolMap[val.toLowerCase()];
}

function verifyRole(val, roles) {
  return; // @TODO
}