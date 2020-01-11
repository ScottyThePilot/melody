'use strict';
const path = require('path');
const { Bot, Command, Logger, utils } = require('./core/core.js');
const config = require('./config.json');
const { exists, mkdir, readdir } = utils.fs;
const { logifyGuild } = utils.logging;

const melody = new Bot({
  config,
  client: {
    disableEveryone: true,
    restTimeOffset: 750,
    disabledEvents: [
      'VOICE_STATE_UPDATE',
      'VOICE_SERVER_UPDATE',
      'TYPING_START',
      'PRESENCE_UPDATE'
    ]
  }
});

melody.init({
  async preInit() {
    this.paths = {
      data: './src/data/',
      commands: './src/commands/'
    };

    if (!await exists(this.paths.data)) await mkdir(this.paths.data);

    this.logger = new Logger(path.join(this.paths.data, 'main.log'), {
      core: path.join(this.paths.data, 'logs'),
      console: true
    });
  },
  async postInit() {
    this.logger.log('INFO', 'Loading Bot...');

    for (let guild of this.client.guilds.values()) {
      await this.loadManager(guild.id);
      this.logger.log('DATA', `Guild ${logifyGuild(guild)} loaded`);
    }

    for (let file of await readdir(this.paths.commands)) {
      const command = require(path.join(this.paths.commands, file.toString()));
      if (command instanceof Command) this.commands.add(command);
    }

    this.logger.log('DATA', `${this.commands.size} Commands loaded`);

    
  }
});

