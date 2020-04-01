import fs from 'fs';
import * as outil from '@utils/obj';

namespace Datastore {
  export interface Options {
    defaultState: object;
    wipeIfCorrupt: boolean;
    compact: boolean;
  }

  export interface Proxy {
    get(p: string | string[]): any;
    set(p: string | string[], value: any): void;
    has(p: string | string[]): boolean;
    write(force: boolean): Promise<boolean>;
  }
}

class Datastore implements Datastore.Proxy {
  ready: boolean;
  synced: boolean;
  readonly path: fs.PathLike;
  private state: object | null;
  private handle: fs.promises.FileHandle;
  private options: Datastore.Options;

  constructor(p: fs.PathLike, handle: fs.promises.FileHandle, options?: Partial<Datastore.Options>) {
    this.options = outil.mergeDefault(Datastore.defaultOptions, options);
    this.handle = handle;
    this.path = p;
    this.state = null;
    this.ready = false;
    this.synced = false;
  }

  static async create(p: fs.PathLike, options?: Partial<Datastore.Options>): Promise<Datastore> {
    return await new Datastore(p, await fs.promises.open(p, 'w+'), options).init();
  }

  private async init(): Promise<this> {
    if (this.ready) throw new Error('Cannot initialize state more than once');

    this.state = await this.resolveState();

    this.ready = true;
    this.synced = true;

    return this;
  }

  get(p: string | string[]): any {
    if (!this.ready) throw new Error('Unable to read/modify state');
    const out = outil.get(this.state as object, p);
    this.synced = false;
    return out;
  }

  set(p: string | string[], value: any) {
    if (!this.ready) throw new Error('Unable to read/modify state');
    outil.set(this.state as object, p, value);
    this.synced = false;
  }

  has(p: string | string[]): boolean {
    if (!this.ready) throw new Error('Unable to read/modify state');
    const out = outil.has(this.state as object, p);
    this.synced = false;
    return out;
  }

  async write(force: boolean = false): Promise<boolean> {
    if (!this.ready) throw new Error('Cannot write state to disk');
    if (this.synced && !force) return false;

    await this.handle.writeFile(this.stringify(this.state), { flag: 'r+' });

    this.synced = true;
    return true;
  }

  async close(write: boolean = false) {
    if (!this.ready) throw new Error('Unable to destroy datastore');

    if (write) await this.write(true);
    await this.handle.close();

    this.ready = false;
    this.synced = false;
    this.state = null;
  }

  private async resolveState(): Promise<object> {
    let data;
    const wipe = this.options.wipeIfCorrupt;
    try {
      data = await this.handle.readFile({ flag: 'r+' });
      if (wipe) data = parseJSON(data);
    } catch {
      data = this.stringify(this.options.defaultState);
      await this.handle.writeFile(data, { flag: 'w+' });
      if (wipe) data = parseJSON(data);
    } finally {
      return wipe ? data : parseJSON(data);
    }
  }

  private stringify(value: any): string {
    return JSON.stringify(value, null, this.options.compact ? 0 : 2);
  }

  static readonly defaultOptions: Datastore.Options = {
    defaultState: {},
    wipeIfCorrupt: true,
    compact: false
  };
}

export default Datastore;

function parseJSON(text: string | Buffer): any {
  return JSON.parse(text.toString());
}
