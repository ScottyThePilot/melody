'use strict';
// Re-Exports
module.exports = {
  Bot: require('./structures/Bot.js'),
  Command: require('./structures/Command.js'),
  GuildManager: require('./structures/GuildManager.js'),
  Logger: require('./structures/Logger.js'),
  Lazystore: require('./structures/Lazystore.js'),

  Collection: require('./structures/Collection.js'),
  Queue: require('./structures/Queue.js'),
  
  utils: {
    fs: require('./modules/utils/fs.js'),
    logging: require('./modules/utils/logging.js'),
    object: require('./modules/utils/object.js')
  }
};
