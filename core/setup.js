'use strict';
const config = require('./config.json');
const util = require('./modules/util/util.js');
const { scheduleJob } = require('node-schedule');
const Blacklist = require('./subfunctions/Blacklist.js');
const CleverBotAgent = require('./subfunctions/CleverBotAgent.js');

const activities = [
  { type: 'WATCHING', name: 'over {server_count} servers' },
  { type: 'WATCHING', name: 'over {user_count} users' },
  { type: 'PLAYING', name: `use ${config.prefix}help` },
  //{ type: 'PLAYING', name: '{global_uptime} days without crashing' },
  { type: 'PLAYING', name: 'after {message_count} messages' },
  { type: 'PLAYING', name: 'after {command_count} commands' },
  { type: 'PLAYING', name: 'for {uptime}' },
  { type: 'PLAYING', name: `in version ${config.version[1]} ${config.version[0]}` },
  { type: 'PLAYING', name: 'Minecraft 2' },
  { type: 'WATCHING', name: 'anime' },
  { type: 'WATCHING', name: 'Scotty\'s lazy ass' },
  { type: 'LISTENING', name: 'existential dread' }
];

module.exports = async function setup(melody) {
  // Subfunctions
  melody.blacklist = new Blacklist(melody.paths.data);
  melody.cleverbot = new CleverBotAgent(30);

  // Scheduled Jobs
  melody.analytics = {
    messages: 0,
    commands: 0,
    pings: [],
    memory: {
      rss: [],
      heapTotal: [],
      heapUsed: []
    }
  };

  melody.jobs = {
    dailyReport: scheduleJob('30 20 * * *', () => dailyReportJob(melody)),
    cycleActivity: scheduleJob('*/20 * * * * *', () => cycleActivityJob(melody)),
    checkLogRotation: scheduleJob('* */2 * * *', () => checkLogRotationJob(melody)),
    collectAnalytics: scheduleJob('*/10 * * * *', () => collectAnalyticsJob(melody))
  };
};

// Daily report sent at 8:30 each day
function dailyReportJob(melody) {
  const ping = util.average(melody.analytics.pings).toFixed(3) + 'ms';
  const rss = util.formatBytes(util.average(melody.analytics.memory.rss));
  const heapTotal = util.formatBytes(util.average(melody.analytics.memory.heapTotal));
  const heapUsed = util.formatBytes(util.average(melody.analytics.memory.heapUsed));

  if (melody.client.status === 0) {
    const owner = melody.client.users.get(config.ownerID);
    const msgText = `**Daily Report:**\nAverage Ping: \`${ping}\`\nAverage Resident Set Size: \`${rss}\`\nAverage Heap Total: \`${heapTotal}\`\nAverage Heap Used: \`${heapUsed}\``;
    owner.send(msgText);
  }

  melody.log(
    'INFO',
    'Daily Report',
    `Average Ping: ${ping}`,
    `Average Resident Set Size: ${rss}`,
    `Average Heap Total: ${heapTotal}`,
    `Average Heap Used: ${heapUsed}`
  );

  melody.analytics.pings = [];
  melody.analytics.memory.rss = [];
  melody.analytics.memory.heapTotal = [];
  melody.analytics.memory.heapUsed = [];
}

// Change client activity randomly every 20 seconds
function cycleActivityJob(melody) {
  if (melody.client.status !== 0) return;
  const msg = activities[Math.floor(Math.random() * activities.length)];
  const uptime = util.formatTime(melody.client.uptime, true);
  const name = msg.name
    .replace('{server_count}', melody.client.guilds.size)
    .replace('{user_count}', melody.client.users.size)
    .replace('{uptime}', uptime)
    .replace('{message_count}', melody.analytics.messages)
    .replace('{command_count}', melody.analytics.commands);
  melody.client.user.setActivity(name, { type: msg.type });
}

// Check log rotation every 2 hours
async function checkLogRotationJob(melody) {
  await melody.logger.checkRotation();
  await util.asyncForEach([...melody.guildManagers.values()], async (manager) => {
    await manager.logger.checkRotation(melody.logger);
  });
}

// Collect analytics data every 10 minutes
function collectAnalyticsJob(melody) {
  let ping = melody.client.ping;
  let { rss, heapTotal, heapUsed } = process.memoryUsage();

  melody.analytics.pings.push(ping);
  melody.analytics.memory.rss.push(rss);
  melody.analytics.memory.heapTotal.push(heapTotal);
  melody.analytics.memory.heapUsed.push(heapUsed);
}
