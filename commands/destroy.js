'use strict';
const Command = require('../modules/Command.js');
const Logger = require('../modules/Logger.js');
const GuildManager = require('../modules/GuildManager.js');
const Util = require('../modules/util/Util.js');
const config = require('../config.json');

async function destroyBot(client) {
  Logger.main.log('INFO', 'Shutting Down...');

  await Util.asyncForEach(client.guilds.array(), async (guild) => {
    await GuildManager.unload(guild.id);
    Logger.main.log('DATA', `Guild ${Logger.logifyGuild(guild)} unloaded`);
  });

  setTimeout(() => {
    Logger.main.end().then(() => {
      client.destroy();
    });
  }, 3000);
}

module.exports = new Command({
  name: 'destroy',
  level: 10,
  plugin: 'core',
  help: {
    short: 'Logs the bot off of Discord.',
    long: 'Logs out, terminates the connection to Discord, and destroys the client.',
    usage: `${config.prefix}destroy`,
    example: `${config.prefix}destroy`
  },
  run: async function (bundle) {
    const { message, client } = bundle;

    message.react(String.fromCharCode(0x2705)).catch();

    await destroyBot(client);
  }
});