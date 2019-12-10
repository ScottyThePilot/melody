'use strict';
const EventEmitter = require('events');

class Queue extends EventEmitter {
  constructor() {
    super();
    this.items = [];
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

  get size() {
    return this.items.length;
  }

  get next() {
    return this.items[0];
  }

  push(item) {
    this.items.push(item);
    this.emit('push');
  }

  pushPromise(item) {
    return new Promise((resolve, reject) => {
      this.push(() => item().then(resolve).catch(reject));
    });
  }

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
