'use strict';
// Types: Calculator, Count, Average

class Analytics {
  constructor(type, ...args) {
    if (type === 'calculator') {
      this.calc = args[1];
    } else if (type === 'count') {
      this.value = args[1] || 0;
    } else if (type === 'average') {
      this.set = args[1] || [];
    }
    this.format = typeof args[2] === 'string' ? args[2] : '{}';
  }

  clear() {
    if (this.type === 'count') {
      this.value = 0;
    } else if (this.type === 'average') {
      this.set = [];
    }
  }

  add(val) {
    if (this.type === 'count') {
      this.value += val || 0;
    } else if (this.type === 'average') {
      if (val !== undefined) this.value.push(val);
    }
  }

  valueOf() {
    if (this.type === 'calculator') return this.calc(this);
    if (this.type === 'count') return this.value;
    if (this.type === 'average') return this.set.reduce((a, c) => a + c) / this.set.length;
  }

  toString() {
    var value = this.type === 'average' ? this.valueOf().toFixed(3) : this.valueOf();
    return this.format.replace(/{}/g, value);
  }

  /*static compare(a, b, format = '{}') {
    var valueOfA = a.valueOf();
    var valueOfB = b.valueOf();
    var compSymbol = (['\u2193', '-', '\u2191'])[Math.sign(valueOfB - valueOfA) + 1];
    return format.replace(/{}/g, valueOfA + ' ' + compSymbol + ' ' + valueOfB);
  }*/

  static getUptime(client) {
    let up = client.uptime;
    let upD = Math.floor(up / 8.64e+7);
    let upH = Math.floor(up / 3.6e+6) % 24;
    let upM = Math.floor(up / 60000) % 60;
    return [upD, upH, upM];
  }

  static phrasifyUptime(client) {
    let [upD, upH, upM] = Analytics.getUptime(client);
    upD += (upD === 1 ? ' day' : ' days');
    upH += (upH === 1 ? ' hour' : ' hours');
    upM += (upM === 1 ? ' minute' : ' minutes');
    return upD + ', ' + upH + ' and ' + upM;
  }

  static updateMemory() {
    var mem = process.memoryUsage();
    Analytics.data.rss.add(mem.rss);
    Analytics.data.heapUsed.add(mem.heapUsed);
    Analytics.data.heapTotal.add(mem.heapTotal);
  }

  static updatePrevAnalytics(client) {
    Analytics.data.prev.users = client.users.size;
    Analytics.data.prev.guilds = client.guilds.size;
  }

  static reset() {
    Analytics.data.messagesSeen.clear();
    Analytics.data.commands.clear();
    Analytics.data.disconnects.clear();
    Analytics.data.ratelimits.clear();
    Analytics.data.rss.clear();
    Analytics.data.heapUsed.clear();
    Analytics.data.heapTotal.clear();
  }

  static setup(client) {
    Analytics.data.uptime.calc = () => Analytics.phrasifyUptime(client);
    Analytics.data.users.calc = () => Analytics.data.prev.users + ' \u2192 ' + client.users.size;
    Analytics.data.guilds.calc = () => Analytics.data.prev.guilds + ' \u2192 ' + client.guilds.size;
    Analytics.data.ping.calc = () => client.ping.toFixed(3);
    Analytics.updatePrevAnalytics(client);
  }
}

Analytics.data = {
  uptime: new Analytics('calculator', null, 'Current Uptime: {}'),

  messagesSeen: new Analytics('count', 0, 'Messages Seen: {}'), // Needs Hook
  commands: new Analytics('count', 0, 'Commands Executed: {}'), // Needs Hook
  users: new Analytics('calculator', null, 'Users: {}'),
  guilds: new Analytics('calculator', null, 'Guilds: {}'),

  ping: new Analytics('calculator', null, 'API Ping: \`{}ms\`'),
  disconnects: new Analytics('count', 0, 'API Disconnects: {}'), // Needs Hook
  ratelimits: new Analytics('count', 0, 'API RateLimits: {}'), // Needs Hook

  rss: new Analytics('average', [], 'Resident Set Size: \`{}mb\`'),
  heap: new Analytics('calculator', () => {
    return Analytics.data.heapUsed.toString() + ' / ' + Analytics.data.heapTotal.toString();
  }, 'Heap: \`{}\`'),
  heapUsed: new Analytics('average', [], '{}mb'),
  heapTotal: new Analytics('average', [], '{}mb'),

  logsRotated: new Analytics('count', 0, 'Logs Rotated: {}'), // Needs Hook

  prev: {
    users: 0,
    guilds: 0
  }
};