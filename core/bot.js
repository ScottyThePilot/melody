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

Logger.main = new Logger('./core/data/main.log', {
  logToConsole: true,
  logPath: './core/data/logs'
});


process.on('unhandledRejection', (reason) => {
  throw reason;
});



client.on('ready', async () => {
  if (!controller.firstReady) {
    Logger.main.log('INFO', 'Bot Loading...');
    
    await wait(750);

    let then = new Date();

    await Util.asyncForEach(client.guilds.array(), async (guild) => {
      await GuildManager.load(guild.id);
      Logger.main.log('DATA', `Guild ${Logger.logifyGuild(guild)} loaded`);
    });

    await Command.buildManifest();

    Logger.main.log('DATA', `${Command.manifest.size} Commands loaded`);

    await client.user.setActivity('in Alpha');

    Logger.main.log('INFO', `Tracking ${client.guilds.size} Guilds with ${client.users.size} Users`);

    controller.firstReady = true;

    Logger.main.log('INFO', 'Bot Ready! (' + (new Date() - then) + 'ms)');
  } else {
    Logger.main.log('INFO', 'Bot Ready!');
  }
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
  let guild = message.guild;

  if (controller.firstReady && guild) {
    let manager = GuildManager.all.get(guild.id);
    if (await manager.configdb.get('logMessageChanges')) controller.onMessage(message, manager);
  }

  if (message.author.bot) return;

  const content = message.content.trim();

  const match = content.match(/^<@!?([0-9]+)>/);
  if (match && match[1] === client.user.id) {
    const msg = content.slice(match[0].length).trim();
    const response = await controller.getCleverBotResponse(msg, message.channel.id);
    await message.channel.send(response, { reply: message.author }).catch(Logger.msgFailCatcher);
    return;
  }

  if (!content.startsWith(config.prefix)) return;

  let args = content.slice(config.prefix.length).split(/\s+/g);
  let command = args.shift().toLowerCase();

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


client.on('guildMemberAdd', async (member) => {
  if (controller.firstReady) controller.onGuildMemberAdd(member, GuildManager.all.get(member.guild.id));
});

client.on('guildMemberRemove', async (member) => {
  if (controller.firstReady) controller.onGuildMemberRemove(member, GuildManager.all.get(member.guild.id));
})

client.on('messageUpdate', async (oldMessage, newMessage) => {
  if (oldMessage.author.bot) return;
  let guild = oldMessage.guild;
  if (controller.firstReady && guild) {
    let manager = GuildManager.all.get(guild.id);
    if (await manager.configdb.get('logMessageChanges')) controller.onMessageUpdate(oldMessage, newMessage, manager);
  }
});

client.on('messageDelete', async (message) => {
  let guild = message.guild;
  if (controller.firstReady && guild) {
    let manager = GuildManager.all.get(guild.id);
    if (await manager.configdb.get('logMessageChanges')) controller.onMessageDelete(message, manager);
  }
});

client.on('messageDeleteBulk', async (messages) => {
  let guild = messages.first().guild;
  if (controller.firstReady, guild) {
    let manager = GuildManager.all.get(guild.id);
    if (await manager.configdb.get('logMessageChanges')) controller.onMessageDeleteBulk(messages, manager);
  }
});



client.on('error', (err) => {
  Logger.main.log('ERR', Logger.logifyError(err));
});


client.on('rateLimit', (err) => {
  let message = `RateLimit ${err.method.toUpperCase()}: ${err.timeDifference}ms (${err.path})`;
  Logger.main.log('WARN', message);
});


client.on('warn', (warn) => {
  Logger.main.log('WARN', warn);
});


client.on('debug', (info) => {
  if (info.includes('Sending a heartbeat')) return;
  if (info.includes('Heartbeat acknowledged, latency of')) return;
  Logger.main.log('DEBUG', 'DiscordDebugInfo: ' + info);
});



client.on('resume', () => {
  Logger.main.log('INFO', 'WebSocket resumed');
});


client.on('reconnecting', () => {
  Logger.main.log('INFO', 'WebSocket reconnecting...');
});


client.on('disconnect', (event) => {
  Logger.main.log('WARN', `WebSocket closed (${event.code})`, event.reason);
  process.exit(0);
});


client.login(config.token);
