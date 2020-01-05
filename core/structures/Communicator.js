'use strict';
const EventEmitter = require('events');

class Communicator extends EventEmitter {
  constructor(proc) {
    super();
    this.process = proc;
    this.process.on('message', (message) => {
      if (!message) return;
      const head = message.head;
      delete message.head;
      this.emit(head, message);
    });
  }

  send(head, message) {
    if (!message) throw new Error('Invalid message');
    message.head = head;
    this.process.send(message);
  }
}

module.exports = Communicator;
