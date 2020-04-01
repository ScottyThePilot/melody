import { EventEmitter } from 'events';

export default class Queue extends EventEmitter {
  private items: Array<() => Promise<any>>;

  constructor() {
    super();

    this.items = [];

    this.on('pop', () => {
      if (this.items.length) this.go();
    });

    this.on('add', () => {
      if (this.items.length === 1) this.go();
    });
  }

  get size(): number {
    return this.items.length;
  }

  private get next(): () => Promise<any> {
    return this.items[0];
  }

  add(item: () => Promise<any>) {
    this.items.push(item);
    this.emit('add');
  }

  wait<T>(item: () => Promise<T>): Promise<T> {
    return new Promise((resolve, reject) => {
      this.add(() => item().then(resolve).catch(reject));
    });
  }

  private async go() {
    const item = this.next;

    try {
      this.emit('start');
      await item();

      this.items.shift();
      this.emit('pop');
    } catch (error) {
      this.emit('error', error);
    }
  }
} 
