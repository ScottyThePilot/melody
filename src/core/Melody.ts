import Command from '@core/Command';
import Manager from '@core/Manager';
import MelodyEvents from '@core/MelodyEvents';
import Logger from '@fs/Logger';
import Group from '@utils/Group';
import Table from '@utils/Table';
import * as tutil from '@utils/text';
import * as util from '@utils/util';
import * as Discord from 'discord.js';
import { EventEmitter } from 'events';
import fs from 'fs';
import path from 'path';

const events = Object.values(Discord.Constants.Events);

/*

*/

namespace Melody {
  export interface Config {
    readonly client?: Discord.ClientOptions;
    readonly version: string;
    readonly token: string;
    readonly prefix: string;
    readonly owner: string;
    readonly trustedUsers: string[];
    readonly paths: {
      readonly data: string,
      readonly commands: string
    };
  }

  export interface Options {
    readonly logger: Logger;
    readonly commands: Group<Command>;
    readonly managers: Table<Discord.Snowflake, Manager>;
    readonly config: Config;
  }
}

class Melody extends EventEmitter implements MelodyEvents, Logger.Proxy {
  ready: boolean;
  readonly client: Discord.Client;
  readonly logger: Logger;
  readonly commands: Group<Command>;
  readonly managers: Table<string, Manager>;
  readonly version: string;
  readonly token: string;
  readonly prefix: string;
  readonly owner: string;
  readonly trustedUsers: string[];
  readonly paths: {
    readonly data: string,
    readonly commands: string
  };

  constructor(client: Discord.Client, options: Melody.Options) {
    super();
    const {
      logger, commands, managers,
      config: { version, token, prefix, owner, trustedUsers, paths }
    } = options;

    this.ready = false;

    this.client = client;
    this.logger = logger;
    this.commands = commands;
    this.managers = managers;

    this.version = version;
    this.token = token;
    this.prefix = prefix;
    this.owner = owner;
    this.trustedUsers = trustedUsers;
    this.paths = paths;
  }

  static async create(config: Melody.Config) {
    const client = new Discord.Client(config.client);

    for (const p of ['data', 'commands'])
      await fs.promises.mkdir(config.paths[p], { recursive: true });
    await fs.promises.mkdir(path.join(config.paths.data, 'guilds'));

    const logger = await Logger.create(path.join(config.paths.data, 'main.log'), {
      logsFolder: path.join(config.paths.data, 'logs'),
      logToConsole: true
    });

    const managers = new Table<string, Manager>();
    const commands = new Group<Command>();

    return await new Melody(client, { logger, managers, commands, config }).init();
  }

  private async init(): Promise<this> {
    await Promise.all([
      util.onceEvent(this.client, 'ready'),
      this.client.login(this.token)
    ]);

    this.log('INFO', 'Connection established');

    await util.wait(1000);

    for (const event of events) {
      if (event === 'message') continue;
      this.client.on(event, (...args) => {
        this.emit(event, ...args);
      });
    }

    this.client.on('message', (message) => {
      const parsed = this.parseCommand(message as Discord.Message);

      if (parsed)
        this.emit('command', parsed);
      else
        this.emit('message', message);
    });

    for (const guild of this.client.guilds.cache.values()) {
      await this.loadManager(guild.id);
      this.logger.log('DATA', `Guild ${tutil.logifyGuild(guild)} loaded`);
    }

    for (const file of await fs.promises.readdir(this.paths.commands)) {
      const location = path.join(this.paths.commands, file.toString());
      this.loadCommandAt(location);
    }

    this.logger.log('DATA', `${this.commands.size} Commands loaded`);

    await this.client.user?.setActivity('waiting...');

    this.logger.log('INFO', 'Bot ready!');

    this.ready = true;

    return this;
  }

  get mention(): RegExp {
    return new RegExp(`^<@!?${(this.client.user as Discord.ClientUser).id}>\\s*`);
  }

  log(header: string, text?: string, ...rest: string[]): boolean {
    return this.logger.log(header, text, ...rest);
  }

  catcher<E extends Error>(error: string | E) {
    const text = error instanceof Error ? tutil.logifyError(error) : error;
    this.logger.log('WARN', 'Caught an error', text);
  }

  async loadManager(id: string) {
    const folder = path.join(this.paths.data, 'guild');
    const manager = await Manager.create(id, folder);
    this.managers.set(id, manager);
  }

  async unloadManager(id: string) {
    const manager = this.managers.get(id);
    if (!manager) throw new Error('Cannot find manager with id ' + id);
    await manager.destroy();
  }

  loadCommandAt(location: string) {
    const command = requireRoot(location);
    if (command instanceof Command)
      this.commands.add(command);
  }

  private parseCommand(message: Discord.Message, prefixOverride?: string): Command.DataBasic | null {
    if (message.author.bot) return null;
  
    const content = message.content.trim();
    const prefix = prefixOverride || this.prefix;
    if (!content.startsWith(prefix)) return null;
  
    // Dissallow whitespace between the prefix and command name
    if (/^\s+/.test(content.slice(prefix.length))) return null;
  
    let args = content.slice(prefix.length).trim().split(/\s+/g);
    if (args.length < 1) return null;

    const command = (args.shift() as string).toLowerCase();
    const argsText = content.slice(prefix.length + command.length).trim();
  
    return { message, command, args, argsText };
  }

  async destroy() {
    await Promise.all([
      this.logger.close(),
      this.managers.valuesGroup().map((m) => m.destroy())
    ])
  }
}

export default Melody;

function requireRoot(id: string) {
  return require(path.join(process.cwd(), id));
}
