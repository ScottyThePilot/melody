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

function wait(ms) {
  return new Promise((res) => setTimeout(res, ms));
}

function capFirst(str) {
  return ''.charAt.call(str, 0).toUpperCase() + ''.slice.call(str, 1).toLowerCase();
}

module.exports = {
  awaitEvent,
  wait,
  capFirst
};
