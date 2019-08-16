'use strict';
// Modules
const Discord = require('discord.js');
const Logger = require('./modules/Logger.js');
const GuildManager = require('./modules/GuildManager.js');
const Command = require('./modules/Command.js');
const Util = require('./modules/util/Util.js');
const controller = require('./modules/controller.js');
const NodeUtil = require('util');

// Constants
const config = require('./config.json');
const client = new Discord.Client({
  disableEveryone: true,
  restTimeOffset: 750,
  disabledEvents: ['VOICE_STATE_UPDATE', 'VOICE_SERVER_UPDATE', 'TYPING_START', 'PRESENCE_UPDATE']
});
const wait = NodeUtil.promisify(setTimeout);


process.on('unhandledRejection', (err) => { 
  throw err; 
});


client.on('ready', async () => {
  if (!controller.firstReady) {
    Logger.main.log('INFO', 'Bot Loading...');

    await wait(750);

    var then = new Date();

    controller.setup(client);

    await Util.asyncForEach(client.guilds.array(), async (guild) => {
      await GuildManager.load(guild.id);
      Logger.main.log('DATA', `Guild ${Logger.logifyGuild(guild)} loaded`);
    });

    await Command.buildManifest();

    Logger.main.log('DATA', `${Command.manifest.size} Commands loaded`);

    await client.user.setActivity('in Beta');

    controller.firstReady = true;
  }

  Logger.main.log('INFO', 'Bot Ready! (' + (new Date() - then) + 'ms)');

  Logger.main.log('INFO', `Tracking ${client.guilds.size} Guilds with ${client.users.size} Users`);
});


client.on('guildCreate', async (guild) => {
  Logger.main.log('INFO', `Guild Found: ${Logger.logifyGuild(guild)}`);
  await GuildManager.load(guild.id);
  Logger.main.log('DATA', `Guild ${Logger.logifyGuild(guild)} loaded`);
});


client.on('guildDelete', async (guild) => {
  Logger.main.log('INFO', `Guild Lost: ${Logger.logifyGuild(guild)}`);
  await GuildManager.unload(guild.id);
  Logger.main.log('DATA', `Guild ${Logger.logifyGuild(guild)} unloaded`);
});


client.on('message', async (message) => {
  if (message.author.bot) return;

  var args = message.cleanContent.trim().slice(config.prefix.length).trim().split(/ +/g);
  var command = args.shift().toLowerCase();

  const found = Command.find(command);

  if (!found) return;

  const bundle = {
    args: args,
    command: command,
    client: client,
    message: message,
    manager: message.guild ? GuildManager.all.get(message.guild.id) : null,
    controller: controller
  };

  await found.attempt(bundle);
});


client.on('error', (err) => {
  Logger.main.log('ERR', Logger.logifyError(err));
});


client.on('rateLimit', (err) => {
  var message = `RateLimit ${err.method.toUpperCase()}: ${err.timeDifference}ms (${err.path})`;
  Logger.main.log('WARN', message);
});


client.on('warn', (warn) => {
  Logger.main.log('WARN', warn);
});


client.login(config.token);


