import Manager from '@core/Manager';
import Melody from '@core/Melody';
import Discord from 'discord.js';

namespace Command {
  export interface DataBasic {
    message: Discord.Message;
    command: string;
    argsText: string;
    args: string[];
  }
  
  export interface Data extends DataBasic {
    melody: Melody;
    level: number;
    location: Command.Location;
    manager: Manager | null;
    message: Discord.Message;
    command: string;
    argsText: string;
    args: string[];
  }

  // Where a command can be
  export type Where = 'dm' | 'guild' | 'anywhere';

  // Where a command is
  export type Location = 'dm' | 'guild';

  export interface Options {
    readonly name: string;
    readonly help: {
      readonly short: string,
      readonly long: string,
      readonly usage: string,
      readonly example: string
    };
    readonly aliases?: string[];
    readonly info?: object;
    readonly where?: Command.Where;
    readonly disabled?: boolean;
    readonly level?: number | string;
    readonly hidden?: boolean;
    readonly run: (this: Command, data: Command.Data) => Promise<void>;
  }
}

class Command {
  readonly name: string;
  readonly help: {
    readonly short: string,
    readonly long: string,
    readonly usage: string,
    readonly example: string
  };
  readonly aliases: string[];
  readonly info: object;
  readonly where: Command.Where;
  readonly disabled: boolean;
  readonly level: number | string;
  readonly hidden: boolean;
  readonly run: (data: Command.Data) => Promise<void>;

  constructor({
    name, help, run,
    aliases = [],
    info = {},
    where = 'anywhere',
    disabled = false,
    level = 0,
    hidden = false
  }: Command.Options) {
    this.name = name;
    this.help = help;
    this.aliases = aliases;
    this.info = info;
    this.where = where;
    this.disabled = disabled;
    this.level = level;
    this.hidden = hidden;
    this.run = run;
  }

  get group(): string {
    if (typeof this.level === 'string')
      return this.level;
    switch (this.level) {
      case 0: return 'Everyone';
      case 1: return 'Server Administrators';
      case 2: return 'Server Owners';
      case 3: return 'Trusted Users';
      case 10: return 'Bot owner';
      default: return 'Unknown';
    }
  }

  async attempt(data: Command.Data): Promise<boolean> {
    if (this.disabled) return false;
    if (!this.here(data.location)) return false;
    if (data.level < this.level) return false;

    await this.run.call(this, data);
    return true;
  }

  here(location: Command.Location): boolean {
    return this.where === 'anywhere' ? true : this.where === location;
  }

  is(query: string) {
    const q = query.toLowerCase();
    if (this.name === q) return true;
    for (const alias of this.aliases)
      if (alias.toLowerCase() === q) return true;
    return false;
  }
}

export default Command;
