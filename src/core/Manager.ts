import Datastore from '@fs/Datastore';
import Logger from '@fs/Logger';
import Queue from '@utils/Queue';
import fs from 'fs';
import path from 'path';

class Manager implements Logger.Proxy, Datastore.Proxy {
  readonly id: string;
  readonly logger: Logger;
  readonly store: Datastore;
  readonly queue: Queue;

  constructor(id: string, logger: Logger, store: Datastore) {
    this.id = id;
    this.logger = logger;
    this.store = store;
    this.queue = new Queue();
  }

  static async create(id: string, location: fs.PathLike, defaultState?: object) {
    const folder: fs.PathLike = path.join(location.toString(), id);
    await fs.promises.mkdir(folder);

    const logger = await Logger.create(path.join(folder, 'latest.log'), {
      logsFolder: path.join(location.toString(), id, 'logs')
    });

    const store = await Datastore.create(path.join(folder, 'store.json'), {
      defaultState
    });

    return new Manager(id, logger, store);
  }

  log(header: string, text?: string, ...rest: string[]): boolean {
    return this.logger.log(header, text, ...rest);
  }

  get(p: string | string[]): any {
    return this.store.get(p);
  }

  set(p: string | string[], value: any) {
    this.store.set(p, value);
  }

  has(p: string | string[]): boolean {
    return this.store.has(p);
  }
  
  write(force: boolean = false): Promise<boolean> {
    return this.queue.wait(() => this.store.write(force));
  }

  async destroy(write: boolean = false) {
    await Promise.all([
      this.logger.close(),
      this.store.close(write)
    ]);
  }
}

export default Manager;
