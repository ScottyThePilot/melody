'use strict';
// Initially written by Yernemm (https://yernemm.xyz)
// Edited by Scotty

const https = require('https');
const Queue = require('./Queue.js');
const md5 = require('./util/cbmd5.js');

class CleverChannel {
  constructor(historyLength = 30) {
    this.msgHistory = [];
    this.msgQueue = new Queue();
    this.historyLength = historyLength;
  }

  saveToHistory(msg) {
    this.msgHistory.unshift(msg);
    while (this.msgHistory.length > this.historyLength) {
      this.msgHistory.pop();
    }
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

  queue(msg) {
    return new Promise((resolve, reject) => {
      this.msgQueue.push(() => {
        return this.send(msg).then(resolve).catch(reject);
      });
    });
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
