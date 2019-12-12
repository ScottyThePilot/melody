'use strict';
const Bot = require('./structures/Bot.js');
const config = require('./config.json');
const events = require('./events.js');
const setup = require('./setup.js');
const util = require('./modules/util.js');

// Crash when a promise rejection goes unhandled
process.on('unhandledRejection', (reason) => {
  throw reason;
});



const melody = new Bot({
  discord: {
    disableEveryone: true,
    restTimeOffset: 750,
    disabledEvents: [
      'VOICE_STATE_UPDATE',
      'VOICE_SERVER_UPDATE',
      'TYPING_START',
      'PRESENCE_UPDATE'
    ]
  },
  paths: {
    data: './core/data',
    commands: './core/commands'
  },
  config
});

// First time setup upon receiving the "ready" event
melody.client.once('ready', async () => {
  melody.log('INFO', 'Bot Loading...');
  
  await util.wait(750);

  const then = new Date();

  for (let guild of melody.client.guilds.values()) {
    await melody.loadGuild(guild.id);
    melody.log('DATA', `Guild ${util.logifyGuild(guild)} loaded`);
  }

  await melody.buildCommands();

  melody.log('DATA', `${melody.commands.size} Commands loaded`);

  await melody.client.user.setActivity('waiting...');

  await setup(melody);

  melody.log('INFO', `Tracking ${melody.client.guilds.size} Guilds with ${melody.client.users.size} Users`);

  melody.ready = true;

  melody.log('INFO', 'Bot Ready! (' + (new Date() - then) + 'ms)');

  melody.log(undefined, `Bot Invite: ${await melody.client.generateInvite(268823760)}`);
});

// Subsequent ready events only log "Bot Ready!"
melody.on('ready', () => {
  melody.log('INFO', 'Bot Ready!');
});



melody.on('guildCreate', async (guild) => {
  melody.log('INFO', `Guild Found: ${util.logifyGuild(guild)}`);
  await melody.loadGuild(guild.id);
  melody.log('DATA', `Guild ${util.logifyGuild(guild)} loaded`);
});

melody.on('guildDelete', async (guild) => {
  melody.log('INFO', `Guild Lost: ${util.logifyGuild(guild)}`);
  await melody.unloadGuild(guild.id);
  melody.log('DATA', `Guild ${util.logifyGuild(guild)} unloaded`);
});



melody.on('message', async (message) => {
  // Log message and continue
  if (message.guild) {
    let manager = melody.guildManagers.get(message.guild.id);
    if (manager.configdb.getSync('logMessages')) {
      events.onMessage(message, manager);
    }
  }

  melody.analytics.messages ++;

  // Exit if bot
  if (message.author.bot) return;

  const content = message.content.trim();

  // Try to match a ping at the beginning of the message
  const match = content.match(/^<@!?([0-9]+)>/);

  // Send CleverBot response and exit if the match was a ping and that ping is the bot
  if (match && match[1] === melody.client.user.id) {
    const msg = content.slice(match[0].length).trim();

    const msgFailCatcher = util.makeCatcher(melody.logger, 'Unable to send message');

    message.channel.startTyping();

    const response = await melody.cleverBot.getResponse(msg, message.channel.id).catch((err) => {
      melody.log('WARN', 'Error while Communicating with CleverBot API: ' + err);
      return 'There was an error while Communicating with the CleverBot API.';
    });

    message.channel.stopTyping();

    if (!response || !response.trim().length) return;

    await message.channel.send(response, { reply: message.author }).catch(msgFailCatcher);
    return;
  }

  // Doesn't start with prefix, ignore message
  if (!content.startsWith(config.prefix)) return;

  let args = content.slice(config.prefix.length).split(/\s+/g);
  let command = args.shift().toLowerCase();

  // Look for command in command list
  const found = melody.findCommand(command);

  // Command not found, ignore message
  if (!found) return;

  // User is blacklisted, ignore message
  if (melody.blacklist.db.getSync().includes(message.author.id)) return;

  const bundle = {
    args,
    command,
    melody,
    message,
    manager: message.guild
      ? melody.guildManagers.get(message.guild.id)
      : null
  };

  // Attempt command
  const result = await found.attempt(bundle, melody.logger);

  if (result.ok) melody.analytics.commands ++;
});



melody.on('guildMemberAdd', async (member) => {
  events.onGuildMemberAdd(member, melody.guildManagers.get(member.guild.id));
});

melody.on('guildMemberRemove', async (member) => {
  events.onGuildMemberRemove(member, melody.guildManagers.get(member.guild.id));
});

melody.on('messageUpdate', async (oldMessage, newMessage) => {
  let guild = oldMessage.guild;
  if (guild) {
    let manager = melody.guildManagers.get(guild.id);
    if (manager.configdb.getSync('logMessageChanges')) {
      events.onMessageUpdate(oldMessage, newMessage, manager);
    }
  }
});

melody.on('messageDelete', async (message) => {
  let guild = message.guild;
  if (guild) {
    let manager = melody.guildManagers.get(guild.id);
    if (manager.configdb.getSync('logMessageChanges')) {
      events.onMessageDelete(message, manager);
    }
  }
});

melody.on('messageDeleteBulk', async (messages) => {
  let guild = messages.first().guild;
  if (guild) {
    let manager = melody.guildManagers.get(guild.id);
    if (manager.configdb.getSync('logMessageChanges')) {
      events.onMessageDeleteBulk(messages, manager);
    }
  }
});



melody.client.on('error', (err) => {
  melody.log('ERR', util.logifyError(err));
});

melody.client.on('rateLimit', (err) => {
  let message = `RateLimit ${err.method.toUpperCase()}: ${err.timeDifference}ms (${err.path})`;
  melody.log('WARN', message);
});

melody.client.on('warn', (warn) => {
  melody.log('WARN', warn);
});

/*melody.client.on('debug', (info) => {
  if (info.startsWith('[ws] [connection] Sending a heartbeat')) return;
  if (info.startsWith('[ws] [connection] Heartbeat acknowledged, latency of')) return;
  if (info.startsWith('READY')) info = 'READY';
  if (info.startsWith('Authenticated using token')) info = 'Authenticated';
  melody.log('DEBUG', 'DiscordDebugInfo: ' + info);
});*/



melody.client.on('resume', () => {
  melody.log('INFO', 'WebSocket resumed');
});

melody.client.on('reconnecting', () => {
  melody.log('INFO', 'WebSocket reconnecting...');
});

melody.client.on('disconnect', (event) => {
  melody.log('INFO', `WebSocket disconnected (${event.code})`, event.reason);
  melody.destroy().then(() => {
    process.exit(0);
  });
});

melody.init().then(() => {
  melody.login().catch(() => {
    melody.log('INFO', 'Unable to log in');
    melody.logger.end().then(() => {
      process.exit(0);
    });
  });
});
