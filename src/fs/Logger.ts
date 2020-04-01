import fs from 'fs';
import path from 'path';
import * as tutil from '@utils/text';
import * as util from '@utils/util';

namespace Logger {
  export interface Options {
    logToConsole: boolean;
    logsFolder: fs.PathLike | null;
    maxFileSize: number;
  }

  export interface Proxy {
    log(header: string, text?: string, ...rest: string[]): boolean;
  }
}

class Logger implements Logger.Proxy {
  ready: boolean;
  readonly path: fs.PathLike;
  private stream: fs.WriteStream;
  private options: Logger.Options;

  constructor(p: fs.PathLike, stream: fs.WriteStream, options?: Partial<Logger.Options>) {
    this.options = util.mergeDefault(Logger.defaultOptions, options);
    this.stream = stream;
    this.path = p;
    this.ready = false;
  }

  static async create(p: fs.PathLike, options?: Partial<Logger.Options>): Promise<Logger> {
    const stream = fs.createWriteStream(p, { flags: 'a' });
    await util.onceEvent(stream, 'ready');
    return await new Logger(p, stream, options).init();
  }

  private async init(): Promise<this> {
    if (this.ready) throw new Error('Cannot initialize state more than once');

    if (this.options.logsFolder !== null) {
      await fs.promises.mkdir(this.options.logsFolder, { recursive: true });
      await this.rotate();
    }

    this.ready = true;

    return this;
  }

  async rotate(): Promise<boolean> {
    if (this.options.logsFolder === null) return false;

    const now = new Date();

    this.stream.cork();

    const handle = await fs.promises.open(this.path, 'r+');
    const { size } = await handle.stat();

    const rotate = size >= this.options.maxFileSize;
    if (rotate) {
      const folder = this.options.logsFolder.toString();
      const filepath = path.join(folder, tutil.savifyDate(now) + '.log');
      
      const contents = await handle.readFile();
      await fs.promises.writeFile(filepath, contents, { flag: 'wx' });
      await handle.writeFile('');
    }

    await handle.close();

    this.stream.uncork();

    return rotate;
  }

  log(header: string, text?: string, ...rest: string[]): boolean {
    const entry = tutil.makeLogEntry(header, text, ...rest);
    if (this.options.logToConsole) console.log(entry);
    return this.stream.writable ? this.stream.write(entry + '\n') : true;
  }

  async close() {
    this.stream.end();
    this.ready = false;
    await util.onceEvent(this.stream, 'finish');
  }

  static readonly defaultOptions: Logger.Options = {
    logToConsole: false,
    logsFolder: null,
    maxFileSize: 524288
  };
}

export default Logger;
