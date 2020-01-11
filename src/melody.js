'use strict';
const path = require('path');
const { Bot, Command, Logger, utils } = require('./core/core.js');
const { exists, mkdir, readdir } = utils.fs;
const { logifyGuild } = utils.logging;

// Crash when a promise rejection goes unhandled
process.on('unhandledRejection', (reason) => {
  let err = new Error(reason);
  err.stack = reason.stack;
  throw err;
});

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
    data: './src/data/',
    guilds: './src/data/guilds',
    commands: './src/commands/'
  }
});

melody.init({
  async preInit() {
    console.log('preinit');
    this.paths = {
      data: './src/data/',
      commands: './src/commands/'
    };

    if (!await exists(this.paths.data))
      await mkdir(this.paths.data);

    this.logger = new Logger(path.join(this.paths.data, 'main.log'), {
      core: path.join(this.paths.data, 'logs'),
      console: true
    });
  },
  async postInit() {
    console.log('loadin');
    this.logger.log('INFO', 'Loading Bot...');

    for (let guild of this.client.guilds.values()) {
      await this.loadManager(guild.id);
      this.logger.log('DATA', `Guild ${logifyGuild(guild)} loaded`);
    }

    if (!await exists(this.paths.commands))
      await mkdir(this.paths.commands);

    for (let file of await readdir(this.paths.commands)) {
      const command = requireRoot(path.join(this.paths.commands, file.toString()));
      if (command instanceof Command) this.commands.add(command);
    }

    this.logger.log('DATA', `${this.commands.size} Commands loaded`);
  }
});

melody.on('message', (message) => {
  console.log('Message: ' + message.content);
});
