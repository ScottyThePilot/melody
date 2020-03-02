'use strict';
// Initially written by Yernemm (https://yernemm.xyz)
// Edited by Scotty
const https = require('https');
const md5 = require('./md5.js');

class CleverBot {
  constructor(historyLength = 30) {
    this.historyLength = historyLength;
    this.msgHistory = [];
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
    let postbody = '';
    postbody += 'stimulus=';
    postbody += encodeURIComponent(msg.trim());
    postbody += this.getHistoryString();
    postbody += '&cb_settings_scripting=no&islearning=1&icognoid=wsf&icognocheck=';
    postbody += md5(postbody.substring(7, 33));
    return postbody;
  }

  send(msg) {
    let postbody = this.getPostbody(msg);
    return new Promise((resolve, reject) => {
      let req = https.request(CleverBot.reqOptions, (res) => {
        res.on('data', (data) => {
          let respMsg = data.toString().split('\r')[0].trim();
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

CleverBot.reqOptions = {
  hostname: 'www.cleverbot.com',
  path: '/webservicemin?uc=UseOfficialCleverbotAPI&',
  method: 'POST',
  headers: {
    'Cookie': 'XVIS=TEI939AFFIAGAYQZ; _cbsid=-1;'
  }
};

module.exports = CleverBot;
