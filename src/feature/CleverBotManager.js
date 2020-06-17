'use strict';
import CleverBot from './CleverBot.js';
import Queue from '../utils/Queue.js';
import Table from '../utils/Table.js';

export default class CleverBotManager {
  /**
   * @param {number} [size=30]
   */
  constructor(size = 30) {
    /** @type {number} @private */
    this.size = size;
    /** @type {Table<string, CleverBotChannel>} @private */
    this.channels = new Table();
  }

  /**
   * @param {string} id
   * @returns {CleverBotChannel | null}
   */
  get(id) {
    return this.channels.has(id)
      ? this.channels.get(id)
      : null;
  }

  /**
   * @param {string} id
   * @returns {Promise<void>}
   */
  clear(id) {
    return this.channels.has(id)
      ? this.channels.get(id).clear()
      : Promise.resolve();
  }

  /**
   * @param {string} id
   * @param {string} msg
   * @returns {Promise<string>}
   */
  send(id, msg) {
    if (!this.channels.has(id))
      this.channels.set(id, new CleverBotChannel(this.size));
    return this.channels.get(id).send(msg);
  }
}

export class CleverBotChannel {
  /**
   * @param {number} [size=30]
   */
  constructor(size = 30) {
    /** @type {CleverBot} @private */
    this.clever = new CleverBot(size);
    /** @type {Queue} @private */
    this.queue = new Queue();
  }

  /**
   * @returns {Promise<void>}
   */
  clear() {
    return this.queue.wait(() => {
      this.clever.clear();
      return Promise.resolve();
    });
  }

  /**
   * @param {string} msg
   * @returns {Promise<string>}
   */
  send(msg) {
    return this.queue.wait(() => this.clever.send(msg));
  }

  /** @type {string[]} */
  get history() {
    return this.clever.history;
  }
}
