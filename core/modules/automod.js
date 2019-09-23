'use strict';

// When a message is sent, if more than this time in ms has elapsed
// since the last message in context, wipe the context before proceeding.
const contextResetThreshold = 30 * 1000;

// Calculates a coeficient to skew the final "heat" score based on
// how many messages the user has sent. Works as a sort of "confidence" value.
function c(x) {
  return 1 - 1 / Math.max(x, 1);
}

// Converts the average delay into a rating which reflects how fast the user
// is sending messages. Higher values mean shorter delays, and vice versa.
function r(x) {
  return 1000 / x;
}

function isMessageSpam(message, manager) {
  if (!manager.autoModContext.has(message.author.id))
    manager.autoModContext.set(message.author.id, []);
  
  const userContext = manager.autoModContext.get(message.author.id);
  const lastTime = userContext[userContext.length - 1];
  const nowTime = +message.createdAt;

  if (userContext.length && nowTime - lastTime > contextResetThreshold) {
    while (userContext.length) userContext.pop();
  }
  
  userContext.push(nowTime);

  if (userContext > 1) {
    // The average delay between messages
    const delaysAvg = userContext
      .map((e, i) => userContext[i - 1] - e)
      .slice(1)
      .reduce((a, c) => a + c)
      / (userContext.length - 1);
    
    // Calculates the final score of how quickly the user is sending messages
    const heat = c(userContext.length) * r(delaysAvg);
    
    // Heat is above the allowed heat threshold
    if (heat > manager.configdb.getSync('autoModSpamThreshold'))
      return true;
  }

  return false;
}

module.exports = {
  antiSpam: {
    isMessageSpam
  }
};