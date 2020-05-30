import { EventEmitter } from 'events';

export default class Queue extends EventEmitter {
  constructor() {
    super();

    /** @type {Array<() => Promise<any>>} */
    this.items = [];

    this.on('pop', () => {
      if (this.items.length) this.go();
    });

    this.on('add', () => {
      if (this.items.length === 1) this.go();
    });
  }

  /** @type {number} */
  get size() {
    return this.items.length;
  }

  /** @type {() => Promise<any>} @private */
  get next() {
    return this.items[0];
  }

  /**
   * @param {() => Promise<any>} item
   */
  add(item) {
    this.items.push(item);
    this.emit('add');
  }

  /**
   * @template T
   * @param {() => Promise<T>} item
   * @returns {Promise<T>}
   */
  wait(item) {
    return new Promise((resolve, reject) => {
      this.add(() => item().then(resolve).catch(reject));
    });
  }

  /**
   * @private
   */
  async go() {
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
