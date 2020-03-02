'use strict';
const path = require('path');
const Bot = require('./core/structures/Bot.js');
const { readdir } = require('./core/modules/utils/fs.js');
const { logifyGuild } = require('./core/modules/utils/logging.js');

// Crash when a promise rejection goes unhandled
process.on('unhandledRejection', (reason) => { throw reason; });

const melody = new Bot({
  config: require('./config.json'),
  client: {
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
    data: './data/',
    guilds: './data/guilds',
    commands: './src/commands/'
  }
});

melody.init(async function () {
  this.logger.log('INFO', 'Loading Bot...');

  for (const guild of this.client.guilds.values()) {
    await this.loadManager(guild.id);
    this.logger.log('DATA', `Guild ${logifyGuild(guild)} loaded`);
  }

  for (const file of await readdir(this.paths.commands)) {
    const location = path.join(this.paths.commands, file.toString());
    await this.loadCommandAt(location);
  }

  this.logger.log('DATA', `${this.commands.size} Commands loaded`);
});

melody.client.on('debug', console.log);

