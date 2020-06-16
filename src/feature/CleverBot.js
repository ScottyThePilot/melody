'use strict';
/* jshint esversion: 9, node: true */
// Initially written by Yernemm (https://yernemm.xyz)
// Edited by ScottyThePilot

import https from 'https';
import md5 from 'md5';

const reqOptions = {
  hostname: 'www.cleverbot.com',
  path: '/webservicemin?uc=UseOfficialCleverbotAPI&',
  method: 'POST',
  headers: {
    'Cookie': 'XVIS=TEI939AFFIAGAYQZ; _cbsid=-1;'
  }
};

function sendCleverBot(msg, history = []) {
  let postbody = '';
  postbody += `stimulus=${encodeURIComponent(msg.trim())}`;
  for (let i = history.length - 1; i >= 0; i --)
    postbody += `&vText${i + 2}=${encodeURIComponent(history[i])}`;
  postbody += `&cb_settings_scripting=no&islearning=1&icognoid=wsf&icognocheck=`;
  postbody += md5(postbody.substring(7, 33));
  
  return new Promise((resolve, reject) => {
    let req = https.request(reqOptions, (res) => {
      res.on('data', (data) => {
        let respMsg = data.toString().split('\r')[0];
        resolve(respMsg);
      });
    });

    req.on('error', reject);
    req.write(postbody);
    req.end();
  });
}

/**
 * A class for managing communication with CleverBot.
 */
export default class CleverBot {
  /**
   * Sends a message directly to CleverBot with a message history array.
   * @param {string} msg The message to send to CleverBot
   * @param {string[]} [history=[]] An optional array of strings representing past messages sent
   * @returns {Promise<string>} A Promise resolving to CleverBot's reply
   */
  static send(msg, history = []) {
    return sendCleverBot(msg, history);
  }

  /**
   * Creates a new CleverBot instance, which automatically manages message history.
   * @param {number} [size=30] The maximum number of messages to keep in history
   * @constructor
   */
  constructor(size = 30) {
    /**
     * The maximum number of messages to keep in history.
     * @type {number}
     * @private
     */
    this.size = size;

    /**
     * An array of strings representing past messages sent to CleverBot.
     * @type {string[]}
     * @private
     */
    this.history = [];
  }

  /**
   * Clears this instance's history array.
   * @returns {this}
   */
  clear() {
    this.history = [];
    return this;
  }

  /**
   * Sends a message to CleverBot.
   * @param {string} msg The message to send to CleverBot
   * @returns {Promise<string>} A Promise resolving to CleverBot's reply
   */
  async send(msg) {
    const reply = await CleverBot.send(msg, this.history);
    this.history.push(msg);
    this.history.push(reply);
    if (this.history.length > this.history)
      this.history.shift();
    return reply;
  }
}
