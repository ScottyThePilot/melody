'use strict';
const EventEmitter = require('events');

class Queue extends EventEmitter {
  constructor() {
    super();

    /** @type {Array<() => Promise>} @private */
    this.items = [];

    /** @type {boolean} */
    this.working = false;

    this.on('pop', () => {
      if (this.items.length) {
        this.go();
        this.working = true;
      } else {
        this.working = false;
      }
    });

    this.on('push', () => {
      if (this.items.length === 1) {
        this.go();
        this.working = true;
      }
    });
  }

  /** @type {number} */
  get size() {
    return this.items.length;
  }

  /** @type {() => Promise} */
  get next() {
    return this.items[0];
  }

  /**
   * Adds a new item to the Queue
   * @param {() => Promise} item 
   */
  push(item) {
    this.items.push(item);
    this.emit('push');
  }

  /**
   * Adds a new item to the Queue, returning a promise
   * resolving upon the item's completion
   * @param {() => Promise} item
   * @returns {Promise}
   */
  pushPromise(item) {
    return new Promise((resolve, reject) => {
      this.push(() => item().then(resolve).catch(reject));
    });
  }

  /**
   * Makes the Queue execute the next item
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

module.exports = Queue;
