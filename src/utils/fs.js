'use strict';
const fs = require('fs');
const { promisify: p } = require('util');

const access = p(fs.access);
const exists = (path, mode) => access(path, mode).then(() => true).catch(() => false);

module.exports = {
  /** @type {(path: string, mode?: number) => Promise<void>} */
  access,

  /** @type {(path: string, mode?: number) => Promise<boolean>} */
  exists,

  /** @type {(path: string, data: any, options?) => Promise<void>} */
  write: p(fs.writeFile),

  /** @type {(path: string, options?) => Promise<Buffer>} */
  read: p(fs.readFile),

  /** @type {(path: string) => Promise<Stats>} */
  stat: p(fs.stat),

  /** @type {(path: string, options?) => Promise<void>} */
  mkdir: p(fs.mkdir),

  /** @type {(path: string, options?) => Promise<void>} */
  rmdir: p(fs.rmdir),

  /** @type {(path: string, options?) => Promise<Buffer[]>} */
  readdir: p(fs.readdir),

  ws: { create, end }
};

/**
 * @param {string} path 
 * @returns {WriteStream}
 */
function create(path) {
  return fs.createWriteStream(path, { flags: 'a' });
}

/**
 * @param {WriteStream} stream 
 * @returns {Promise<void>}
 */
function end(stream) {
  return new Promise((resolve, reject) => {
    stream.end((err) => err ? reject(err) : resolve());
  });
}

/**
 * @typedef {import('fs').WriteStream} WriteStream
 * @typedef {import('fs').Stats} Stats
 */
