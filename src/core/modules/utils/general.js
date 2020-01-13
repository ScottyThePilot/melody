'use strict';

/**
 * @param {import('events').EventEmitter} emitter 
 * @param {string} event
 * @returns {Promise}
 */
function awaitEvent(emitter, event) {
  return new Promise((resolve) => {
    emitter.once(event, resolve);
  });
}

module.exports = {
  awaitEvent
};
