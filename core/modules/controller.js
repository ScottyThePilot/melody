'use strict';
const Logger = require('./Logger.js');
const GuildManager = require('./GuildManager.js');
const Command = require('./Command.js');
const Util = require('./util/Util.js');
const CleverChannel = require('./CleverChannel.js');
const Datastore = require('./Datastore.js');
const { scheduleJob } = require('node-schedule');
const { version, ownerID } = require('../config.json');

const cleverChannels = new Map();
const blacklist = new Datastore('./core/data/blacklist.json', {
  defaultData: [],
  persistence: true
});

const jobs = {};

const analytics = {
  messages: 0,
  commands: 0,
  pings: [],
  memory: {
    rss: [],
    heapTotal: [],
    heapUsed: []
  }
};

const activities = [
  { type: 'WATCHING', name: 'over {server_count} servers' },
  { type: 'WATCHING', name: 'over {user_count} users' },
  { type: 'PLAYING', name: 'use ;help' },
  //{ type: 'PLAYING', name: '{global_uptime} days without crashing' },
  //{ type: 'PLAYING', name: 'after {message_count} messages' },
  { type: 'PLAYING', name: 'for {uptime}' },
  { type: 'PLAYING', name: `in version ${version[1]} ${version[0]}` },
  { type: 'PLAYING', name: 'Minecraft 2' },
  { type: 'WATCHING', name: 'anime' },
  { type: 'WATCHING', name: 'Scotty\'s lazy ass' },
  { type: 'LISTENING', name: 'existential dread' }
];


function average(arr) {
  return arr.reduce((a, c) => a + c, 0) / arr.length;
}

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

function onGuildMemberAdd(member, manager) {
  manager.log('LOGGER', `User ${Logger.logifyUser(member)} added to guild`);
}

function onGuildMemberRemove(member, manager) {
  manager.log('LOGGER', `User ${Logger.logifyUser(member)} removed from guild`);
}

function onMessageUpdate(oldMessage, newMessage, manager) {
  const oldContent = `Old Content: \"${Logger.escape(Logger.cleanContent(oldMessage))}\"`;
  const oldMeta = Logger.stylizeMetaData(oldMessage).map((e) => '  ' + e);
  const newContent = `New Content: \"${Logger.escape(Logger.cleanContent(newMessage))}\"`;
  const newMeta = Logger.stylizeMetaData(newMessage).map((e) => '  ' + e);
  manager.log('LOGGER', `Message by user ${Logger.logifyUser(oldMessage.author)} edited in channel ${Logger.logify(oldMessage.channel)}`, oldContent, ...oldMeta, newContent, ...newMeta);
}

function onMessageDelete(message, manager) {
  const content = `Content: \"${Logger.escape(Logger.cleanContent(message))}\"`;
  const meta = Logger.stylizeMetaData(message).map((e) => '  ' + e);
  manager.log('LOGGER', `Message by user ${Logger.logifyUser(message.author)} deleted in channel ${Logger.logify(message.channel)}`, content, ...meta);
}

function onMessageDeleteBulk(messages, manager) {
  const list = messages.array().map((message) => {
    const header = `Message by user ${Logger.logifyUser(message.author)}:`;
    const content = `  Content: \"${Logger.escape(Logger.cleanContent(message))}\"`;
    const meta = Logger.stylizeMetaData(message).map((e) => '    ' + e);
    return [header, content, ...meta];
  });
  manager.log('LOGGER', `Bulk message deletion in channel ${Logger.logify(messages.first().channel)}`, ...[].concat(...list));
}

function onMessage(message, manager) {
  const content = `Content: \"${Logger.escape(Logger.cleanContent(message))}\"`;
  const meta = Logger.stylizeMetaData(message).map((e) => '  ' + e);
  manager.log('LOGGER', `Message by user ${Logger.logifyUser(message.author)} sent in channel ${Logger.logify(message.channel)}`, content, ...meta);
}

function userOwnsAGuild(user, client) {
  return client.guilds.some((guild) => guild.owner.id === user.id);
}

async function getCleverBotResponse(msg, ch) {
  if (!cleverChannels.has(ch)) cleverChannels.set(ch, new CleverChannel(25));
  const channel = cleverChannels.get(ch);
  return await channel.send(msg).catch((err) => {
    Logger.main.log('WARN', 'Error while Communicating with CleverBot API: ' + err.message);
  });
}

async function blacklistAdd(user) {
  let out;
  await blacklist.transform((data) => {
    if (!data.includes(user.id)) {
      data.push(user.id);
      out = true;
    } else {
      out = false;
    }
    return data;
  });
  return out;
}

async function blacklistRemove(user) {
  let out;
  await blacklist.transform((data) => {
    if (data.includes(user.id)) {
      data.splice(data.indexOf(user.id), 1);
      out = true;
    } else {
      out = false;
    }
    return data;
  });
  return out;
}

function resolveUser(val, client) {
  if (!val || typeof val !== 'string' || !val.trim().length) return null;
  if (client.users.has(val.trim())) return client.users.get(val.trim());
  const match = val.trim().match(/[0-9]+/);
  if (!match) return null;
  return client.users.get(match[0]) || null;
}

function setup(client) {
  // Daily report sent at 7:15
  jobs.dailyReport = scheduleJob('30 20 * * *', () => {
    const owner = client.users.get(ownerID);

    const ping = average(analytics.pings).toFixed(3);
    const rss = Logger.logifyBytes(average(analytics.memory.rss));
    const heapTotal = Logger.logifyBytes(average(analytics.memory.heapTotal));
    const heapUsed = Logger.logifyBytes(average(analytics.memory.heapUsed));

    owner.send(`**Daily Report:**\nAverage Ping: \`${ping}ms\`\nAverage Resident Set Size: \`${rss}\`\nAverage Heap Total: \`${heapTotal}\`\nAverage Heap Used: \`${heapUsed}\``);

    analytics.pings = [];
    analytics.memory.rss = [];
    analytics.memory.heapTotal = [];
    analytics.memory.heapUsed = [];
  });

  // Change client activity randomly every 20 seconds
  jobs.cycleActivity = scheduleJob('*/20 * * * * *', () => {
    const msg = activities[Math.floor(Math.random() * activities.length)];
    const uptime = Logger.getUptime(client);
    const name = msg.name
      .replace('{server_count}', client.guilds.size)
      .replace('{user_count}', client.users.size)
      .replace('{uptime}', `${uptime[0]}d, ${uptime[1]}h, and ${uptime[2]}m`);
    client.user.setActivity(name, { type: msg.type });
  });

  // Check log rotation every 2 hours
  jobs.checkLogRotation = scheduleJob('* */2 * * *', async () => {
    await Logger.main.checkRotation();
    await Util.asyncForEach([...GuildManager.all.values()], async (manager) => {
      await manager.logger.checkRotation();
    });
  });

  // Collect analytics data every 10 minutes
  jobs.collectAnalytics = scheduleJob('*/10 * * * *', () => {
    let ping = client.ping;
    let { rss, heapTotal, heapUsed } = process.memoryUsage();

    analytics.pings.push(ping);
    analytics.memory.rss.push(rss);
    analytics.memory.heapTotal.push(heapTotal);
    analytics.memory.heapUsed.push(heapUsed);
  });
}

function getLifetime() {
  process.send({
    head: 'info.lifetime',
    type: 'request'
  });

  return new Promise(function (resolve) {
    process.on('message', function handler(message) {
      if (message.head === 'info.lifetime' && message.type === 'response') {
        resolve(message.data);
      }
    });
  });
}

function getDataTree() {
  function getDirStructure(path) {
    
  }
}


module.exports = {
  destroyBot,
  getAccessiblePlugins,
  userOwnsAGuild,
  getCleverBotResponse,
  getLifetime,

  onGuildMemberAdd,
  onGuildMemberRemove,
  onMessageUpdate,
  onMessageDelete,
  onMessageDeleteBulk,
  onMessage,

  blacklistAdd,
  blacklistRemove,

  resolveUser,

  setup,

  cleverChannels,
  blacklist,
  jobs,
  analytics,

  firstReady: false
};
