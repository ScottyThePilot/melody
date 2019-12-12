'use strict';
const Command = require('../structures/Command.js');
const config = require('../config.json');
const util = require('../modules/util.js');
const Result = require('../structures/Result.js');

const {
  CFG_INVALID_SUBCOMMAND,
  CFG_PROP_DESCRIPTIONS: descriptions,
  BOOLEAN_KEYWORDS: configBoolMap
} = require('../modules/constants.js');

const invalidSubcommandMessage = util.format(CFG_INVALID_SUBCOMMAND, config.prefix);

const configProperties = {
  logMessages: {
    desc: util.format(descriptions.logMessages, config.prefix),
    set: setBoolProp('logMessages'),
    get: getBoolProp('logMessages')
  },
  logMessageChanges: {
    desc: util.format(descriptions.logMessageChanges, config.prefix),
    set: setBoolProp('logMessageChanges'),
    get: getBoolProp('logMessageChanges')
  },
  mutedRole: {
    desc: util.format(descriptions.mutedRole, config.prefix),
    set: setMutedRole,
    get: getMutedRole
  }
};

const propList = util.listify(Object.keys(configProperties).map((p) => `\`${p}\``));

const configNoneList = ['none', 'null', 'disable', 'disabled'];

module.exports = new Command({
  name: 'configure',
  level: 2,
  plugin: 'core',
  help: {
    short: 'Changes server config settings.',
    long: 'Allows the server owner to modify server configuration settings for the bot. Exclude the \`value\` argument to get the current value of a property or exclude the \`config property\` argument to list all valid properties.',
    usage: `${config.prefix}configure <'list'|'get'|'set'> [config property]`,
    example: `${config.prefix}configure set logMessageChanges true`
  },
  aliases: ['config', 'cfg'],
  inDM: false,
  run: async function run({ melody, message, manager, args }) {
    const msgFailCatcher = util.makeCatcher(melody.logger, 'Unable to send message');

    if (!args[0]) {
      await message.channel.send(invalidSubcommandMessage).catch(msgFailCatcher);
    } else if (args[0].toLowerCase() === 'list') {
      await message.channel.send(`Valid config properties are:\n${propList}.`).catch(msgFailCatcher);
    } else if (args[0].toLowerCase() === 'get') {
      const propName = getConfigProp(args[1]);
      const prop = configProperties[propName];

      if (!args[1]) {
        await message.channel.send('Please provide a valid config property that you\'d like to me to retrieve the value of.').catch(msgFailCatcher);
      } else if (!prop) {
        await message.channel.send('That is not a valid config property.').catch(msgFailCatcher);
      } else {
        const value = prop.get(melody, manager);

        await message.channel.send(`The value of \`${propName}\` is currently \`${value}\`.\n${prop.desc}`).catch(msgFailCatcher);
      }
    } else if (args[0].toLowerCase() === 'set') {
      const propName = getConfigProp(args[1]);
      const prop = configProperties[propName];

      if (!args[1]) {
        await message.channel.send('Please provide a valid config property along with a value you wish to be set.').catch(msgFailCatcher);
      } else if (!prop) {
        await message.channel.send('That is not a valid config property.').catch(msgFailCatcher);
      } else {
        const result = await prop.set(melody, manager, args[2]);

        if (result.ok) {
          await message.channel.send(`The value of \`${propName}\` has been updated to \`${result.value}\`.`).catch(msgFailCatcher);
        } else {
          await message.channel.send(result.error).catch(msgFailCatcher);
        }
      }
    } else {
      await message.channel.send(invalidSubcommandMessage).catch(msgFailCatcher);
    }
  }
});

function getConfigProp(name) {
  if (!name) return null;
  const clean = name.replace(/[^a-zA-Z]/g, '').toLowerCase();
  for (let key in configProperties) {
    if (key.toLowerCase() === clean) return key;
  }
  return null;
}

function formatRole(role) {
  return role ? util.logify(role) : '[None]';
}



function setBoolProp(prop) {
  return async function (_melody, manager, value) {
    const valid = configBoolMap.hasOwnProperty(value.toLowerCase());
    if (!valid) return new Result.Err('That is not a valid value. Try values like \`true\` or \`false\`.');

    const cleanValue = configBoolMap[value.toLowerCase()];

    await manager.configdb.set(prop, cleanValue);
    manager.log('DATA', `Config property ${prop} updated to ${cleanValue}`);
    return new Result.Ok(cleanValue);
  };
}

function getBoolProp(prop) {
  return function (_melody, manager) {
    return manager.configdb.getSync(prop);
  };
}



async function setMutedRole(melody, manager, value) {
  const disable = configNoneList.includes(value.toLowerCase());
  const guild = melody.client.guilds.get(manager.id);
  const resolved = util.resolveGuildRole(guild, value);
  const valid = disable || resolved !== null;

  if (!valid) return new Result.Err('I could not find a valid role with that ID/name.');

  const cleanValue = disable ? null : resolved.id;

  await manager.configdb.set('mutedRole', cleanValue);
  manager.log('DATA', `Config property mutedRole updated to ${cleanValue}`);
  return new Result.Ok(formatRole(resolved));
}

function getMutedRole(melody, manager) {
  const value = manager.configdb.getSync('mutedRole');
  const guild = melody.client.guilds.get(manager.id);
  return formatRole(util.resolveGuildRole(guild, value));
}
