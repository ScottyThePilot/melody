'use strict';
// Modules
const Discord = require('discord.js');
const Logger = require('./modules/Logger.js');
const GuildManager = require('./modules/GuildManager.js');
const NodeUtil = require('util');
const Util = require('./modules/util/Util.js');

// Constants
const config = require('./config.json');
const client = new Discord.Client({
  disableEveryone: true,
  restTimeOffset: 750,
  disabledEvents: ['VOICE_STATE_UPDATE', 'VOICE_SERVER_UPDATE', 'TYPING_START', 'PRESENCE_UPDATE']
});
const wait = NodeUtil.promisify(setTimeout);

// Terminate on Unhandled Promise Rejection
process.on('unhandledRejection', (err) => { 
  throw err; 
});

// Ready
client.on('ready', async () => {
  Logger.main.log('INFO', 'Bot Loading...');

  await wait(1000); // Wait

  await client.user.setActivity(`Use ${config.prefix}help`); // Set Activity

  Logger.main.log('INFO', 'Bot Ready!');

  Logger.main.log('INFO', `Tracking ${client.guilds.size} Guilds with ${client.users.size} Users`);

  await Util.asyncForEach(client.guilds.array(), async (guild) => {
    await GuildManager.load(guild.id);
    Logger.main.log('DATA', `Guild ${Logger.logifyGuild(guild)} loaded`);
  });

  // Temporary
  wait(10000).then(() => {
    Logger.main.log('INFO', 'Shutting Down...');
    client.destroy();
  });
});

client.on('guildCreate', async (guild) => {
  Logger.main.log('INFO', `Guild Found: ${Logger.logifyGuild(guild)}`);
  await GuildManager.load(guild.id);
  Logger.main.log('DATA', `Guild ${Logger.logifyGuild(guild)} loaded`);
});

client.on('guildDelete', async (guild) => {
  Logger.main.log('INFO', `Guild Found: ${Logger.logifyGuild(guild)}`);
  await GuildManager.unload(guild.id);
  Logger.main.log('DATA', `Guild ${Logger.logifyGuild(guild)} unloaded`);
});

// Message
client.on('message', async (message) => {
  if (message.author.bot) return;

  var args = message.cleanContent.trim().slice(config.prefix.length).trim().split(/ +/g);
  var command = args.shift().toLowerCase();

  const bundle = {
    client: client,
    config: config,
    args: args,
    message: message,
    manager: GuildManager.all.get(message.guild.id),
  };

  console.log(message.toString());
});

// Send ERR log on client.error
client.on('error', (err) => {
  Logger.main.log('ERR', err.message);
});

// Send WARN log on client.rateLimit
client.on('rateLimit', (err) => {
  var message = `RateLimit ${err.method.toUpperCase()}: ${err.timeDifference}ms (${err.limit})`;
  Logger.main.log('WARN', message);
});

// Send WARN log on client.warn
client.on('warn', (warn) => {
  Logger.main.log('WARN', warn);
});

Logger.main.log(undefined, '[Begin Log]');

client.login(config.token);


