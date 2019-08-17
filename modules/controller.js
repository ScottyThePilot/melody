'use strict';
const Logger = require('./Logger.js');
const GuildManager = require('./GuildManager.js');
const Command = require('./Command.js');
const Util = require('./util/Util.js');
const controller = {};

async function destroyBot(client) {
  Logger.main.log('INFO', 'Shutting Down...');

  await Util.asyncForEach(client.guilds.array(), async (guild) => {
    await GuildManager.unload(guild.id);
    Logger.main.log('DATA', `Guild ${Logger.logifyGuild(guild)} unloaded`);
  });

  await Logger.main.end();
  
  await client.destroy();
}

async function getAccessiblePlugins(user, client) {
  let userPlugins = Command.pluginsDM.slice(0);

  await Util.asyncForEach([...GuildManager.all.values()], async (manager) => {
    let guild = client.guilds.get(manager.id);

    if (!guild.members.has(user.id)) return;

    let plugins = await manager.configdb.get('plugins');

    plugins.forEach((plugin) => {
      if (!userPlugins.includes(plugin)) userPlugins.push(plugin);
    });
  });

  return userPlugins;
}

controller.setup = function setup(client) {
  controller.destroyBot = () => destroyBot(client);
  controller.userOwnsAGuild = (user) => client.guilds.some((guild) => guild.owner.id === user.id);
  controller.getAccessiblePlugins = (user) => getAccessiblePlugins(user, client);
};

controller.firstReady = false;

module.exports = controller;