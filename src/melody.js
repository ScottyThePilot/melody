'use strict';
const path = require('path');
const Bot = require('./core/Bot.js');
const Command = require('./core/Command.js');
const { readdir } = require('./utils/fs.js');
const { logifyGuild } = require('./utils/text.js');

// Crash when a promise rejection goes unhandled
process.on('unhandledRejection', (reason) => { throw reason; });

class Melody extends Bot {
  constructor() {
    super({
      config: require('./config.json'),
      client: require('./client_config.json'),
      paths: {
        data: './data/',
        guilds: './data/guilds',
        commands: './src/commands/'
      }
    });

    /** @type {boolean} */
    this.ready = false;

    this.embeds = {
      help: {
        plugins: [],
        commands: []
      },
      changelog: null
    };

    //this.client.on('debug', console.log);
    this.on('command', (...args) => this.onCommand(...args));
    this.on('message', (...args) => this.onMessage(...args));
  }

  async init() {
    await super.init();

    this.logger.log('INFO', 'Connection established');
  
    for (const guild of this.client.guilds.values()) {
      await this.loadManager(guild.id);
      this.logger.log('DATA', `Guild ${logifyGuild(guild)} loaded`);
    }
  
    for (const file of await readdir(this.paths.commands)) {
      const location = path.join(this.paths.commands, file.toString());
      await this.loadCommandAt(location);
    }
  
    this.logger.log('DATA', `${this.commands.size} Commands loaded`);

    await this.client.user.setActivity('waiting...');

    this.ready = true;

    this.logger.log('INFO', `Tracking ${this.client.guilds.size} Guilds with ${this.client.users.size} Users`);
    this.logger.log(undefined, `Bot Invite: ${await this.client.generateInvite(268823760)}`);
    this.logger.log('INFO', 'Bot ready!');
  }

  catcher(error) {
    this.logger.log('WARN', 'Caught an error', error);
  }

  async onCommand(data) {
    const cmd = this.commands.find((c) => c.is(data.command));

    if (!cmd) return;

    return await cmd.attempt({
      melody: this,
      level: this.getUserLevel(data),
      where: data.message.guild ? 'Guild' : 'DM',
      manager: data.message.guild
        ? this.managers.get(data.message.guild.id)
        : null,
      ...data
    });
  }

  async onMessage(message) {
    
  }

  getUserLevel({ message }) {
    let userLevel = 0;

    if (message.guild) {
      if (message.member.hasPermission('ADMINISTRATOR')) userLevel = 1;
      if (message.guild.owner.id === message.author.id) userLevel = 2;
    } else if (this.trustedUsers.includes(message.author.id)) userLevel = 3;
  
    if (this.owner === message.author.id) userLevel = 10;

    return userLevel;
  }
}

const melody = new Melody();
melody.init();
