'use strict';
// Initially written by Yernemm (https://yernemm.xyz)
// Edited by Scotty

const https = require('https');
const Queue = require('./Queue.js');
const md5 = require('../modules/cbmd5.js');

class CleverChannel {
  constructor(historyLength = 30, queueSizeLimit = 10) {
    if (queueSizeLimit < 0) queueSizeLimit = Infinity;
    this.historyLength = historyLength;
    this.queueSizeLimit = queueSizeLimit;
    this.msgHistory = [];
    this.msgQueue = new Queue();
  }

  saveToHistory(msg) {
    this.msgHistory.unshift(msg);
    if (this.msgHistory.length > this.historyLength)
      this.msgHistory.pop();
  }

  clearHistory() {
    this.msgHistory = [];
  }

  getHistoryString() {
    return this.msgHistory.map((item, i) => {
      return `&vText${i + 2}=${encodeURIComponent(item)}`;
    }).join('');
  }

  getPostbody(msg) {
    let postbody = `stimulus=${encodeURIComponent(msg.trim())}${this.getHistoryString()}&cb_settings_scripting=no&islearning=1&icognoid=wsf&icognocheck=`;
    postbody += md5(postbody.substring(7, 33));
    return postbody;
  }

  async queue(msg) {
    if (this.msgQueue.size >= this.queueSizeLimit) return null;
    return await this.msgQueue.pushPromise(() => this.send(msg));
  }

  send(msg) {
    if (!msg.trim().length) return Promise.resolve();
    let postbody = this.getPostbody(msg);
    return new Promise((resolve, reject) => {
      let req = https.request(CleverChannel.reqOptions, (res) => {
        res.on('data', (data) => {
          let respMsg = data.toString().split('\r')[0];
          this.saveToHistory(respMsg);
          resolve(respMsg);
        });
      });

      req.on('error', reject);
      req.write(postbody);
      req.end();
    });
  }
}

CleverChannel.reqOptions = {
  hostname: 'www.cleverbot.com',
  path: '/webservicemin?uc=UseOfficialCleverbotAPI&',
  method: 'POST',
  headers: {
    'Cookie': 'XVIS=TEI939AFFIAGAYQZ; _cbsid=-1;'
  }
};

module.exports = CleverChannel;
