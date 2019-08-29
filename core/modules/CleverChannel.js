// Initially written by Yernemm (https://yernemm.xyz)
// Edited by Scotty
'use strict';
const https = require('https');
const md5 = require('./util/cb_md5.js');

class CleverChannel {
  constructor(historyLength = 30) {
    this.msgHistory = [];
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
    var postbody = `stimulus=${encodeURIComponent(msg)}${this.getHistoryString()}&cb_settings_scripting=no&islearning=1&icognoid=wsf&icognocheck=`;
    postbody += md5(postbody.substring(7, 33));
    return postbody;
  }

  send(msg) {
    var postbody = this.getPostbody(msg);
    var that = this;
    return new Promise((resolve, reject) => {
      var req = https.request(CleverChannel.postUrl, CleverChannel.reqOptions, (res) => {
        res.on('data', (data) => {
          var respMsg = data.toString().split('\r')[0];
          that.saveToHistory(respMsg);
          resolve(respMsg);
        });
      });

      req.on('error', reject);
      req.write(postbody);
      req.end();
    });
  }
}

CleverChannel.postUrl = 'https://www.cleverbot.com/webservicemin?uc=UseOfficialCleverbotAPI&';

CleverChannel.reqOptions = {
  method: 'POST',
  headers: {
    'Cookie': 'XVIS=TEI939AFFIAGAYQZ; _cbsid=-1;'
  }
};

module.exports = CleverChannel;