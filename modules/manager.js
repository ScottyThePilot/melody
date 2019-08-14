'use strict';
const Logger = require('./Logger.js');
const GuildManager = require('./GuildManager.js');
const manager = {};

async function destroyBot(client) {
  Logger.main.log('INFO', 'Shutting Down...');

  await Util.asyncForEach(client.guilds.array(), async (guild) => {
    await GuildManager.unload(guild.id);
    Logger.main.log('DATA', `Guild ${Logger.logifyGuild(guild)} unloaded`);
  });

  Logger.main.end().then(() => {
    client.destroy();
  });
}

manager.setup = function setup(client) {
  manager.destroyBot = destroyBot.bind(manager, client);
  manager.shutDownBot
};

module.exports = manager;