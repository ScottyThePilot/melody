'use strict';
const Logger = require('./Logger.js');
const GuildManager = require('./GuildManager.js');
const Util = require('./util/Util.js');
const controller = {};

async function destroyBot(client) {
  Logger.main.log('INFO', 'Shutting Down...');

  await Util.asyncForEach(client.guilds.array(), async (guild) => {
    await GuildManager.unload(guild.id);
    Logger.main.log('DATA', `Guild ${Logger.logifyGuild(guild)} unloaded`);
  });

  return Logger.main.end().then(() => {
    client.destroy();
  });
}

controller.setup = function setup(client) {
  controller.destroyBot = () => destroyBot(client);
};

module.exports = controller;