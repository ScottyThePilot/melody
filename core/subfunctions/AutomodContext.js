'use strict';

class AutomodContext {
  constructor(configdb) {
    this.configdb = configdb;
    this.map = new Map();
  }

  get enabled() {
    return this.configdb.getSync('automod');
  }

  get threshold() {
    return this.configdb.getSync('automodThreshold');
  }

  /**
   * Tells this context that a user sent a message.
   * @param {Discord#Message} message A message that was sent
   * @returns {Boolean} Whether the message triggered anti-spam or not
   */
  sent(message) {
    if (!this.map.has(message.author.id))
      this.map.set(message.author.id, new UserContext());
    
    const ctx = this.map.get(message.author.id);

    ctx.timestamps.push(message.createdTimestamp);
    const triggered = ctx.detect(message.createdTimestamp, this.threshold);

    if (triggered) ctx.triggerTimestamp = message.createdTimestamp;

    return triggered;
  }
}

class UserContext {
  constructor() {
    this.triggerTimestamp = null;
    this.timestamps = [];
  }

  detect(now, threshold) {
    // Remove timestamps that are too old
    while (now - this.timestamps[0] > UserContext.timestampMaxAge)
      this.timestamps.shift();
    
    // Remove extra timestamps
    while (this.timestamps.length > UserContext.timestampMaxLength)
      this.timestamps.shift();
  
    const { timestamps, triggerTimestamp: trigger } = this;
    
    // Exit if there are no timestamps left
    if (timestamps.length <= 1) return false; 
    
    // Variable to modify threshold if the user has triggered automod recently
    const suspicious = trigger && trigger < this.triggerStampMaxAge;
    if (!suspicious) this.triggerTimestamp = null;
  
    const s = suspicious ? UserContext.repeatPenalty : 1;
    const c = UserContext.confidence(timestamps);
    
    // Determine the average rate (messages per sec) the user is sending
    const rate = 1000 / average([...getDelays(timestamps)]);
    const thresholdAdjusted = threshold * s * c;
  
    return rate > thresholdAdjusted;
  }

  static confidence(timestamps) {
    return 9 / timestamps.length ** 2 + 1;
  }
}

UserContext.timestampMaxAge = 60e4;
UserContext.timestampMaxLength = 7;
UserContext.triggerStampMaxAge = 360e4;
UserContext.repeatPenalty = 0.9;

AutomodContext.UserContext = UserContext;

module.exports = AutomodContext;

function average(arr) {
  let sum = 0;
  for (let a of arr) sum += a;
  return sum / arr.length;
}

function * getDelays(timestamps) {
  let last;
  for (let time of timestamps) {
    if (last !== undefined) yield time - last;
    last = time;
  }
}
