'use strict';
const fs = require('fs');
const crypto = require('crypto');
const salt = Buffer.from('2fd43724ed0da6d3e0bb63ff0b16e527', 'hex');

/**
 * Executes a list of AsyncFunctions (or Functions returning Promises) one at a time.
 */
class Sequencer {
  /**
   * Creates a Sequencer instance.
   * @param {Array} items An array of AsyncFunctions (or Functions returning Promises)
   */
  constructor(items = []) {
    this.items = items;
    this.next();
  }

  /**
   * Adds an AsyncFunction or Function to the item list.
   * @param {AsyncFunction|Function} fn The Function to be added to the item list
   */
  push(fn) {
    this.items.push(fn);
    if (!this.waiting) this.next();
  }

  /**
   * Used internally to handle execution.
   * @private
   */
  next() {
    if (!this.items.length) {
      this.waiting = false;
      return;
    }
    this.waiting = true;
    const that = this;
    this.items[0]().then(function () {
      that.items.shift();
      that.next();
    });
  }
}

const defaultOptions = {
  spacing: true,
  encrypt: false,
  key: null,
  data: {}
};

/**
 * A class that manages the reading and writing of data to a file.
 */
class Datastore {
  /**
   * Creates a Datastore instance.
   * @param {String} path The path to the file
   * @param {Object} [options] Options specifying further behavior of the Datastore
   * @param {Boolean} [options.spacing=true] Whether or not the JSON in files should be spaced or not
   * @param {Boolean} [options.encrypt=false] Whether or not the JSON in the files should be encrypted or not
   * @param {*} [options.key] The encryption key to be used if encryption is enabled
   * @param {*} [options.data={}] Data to be written if the file does not exist
   */
  constructor(path, options) {
    /**
     * The path to the file
     * @type {String}
     */
    this.path = path;

    /**
     * This Datastore's Sequencer, managing file operations
     * @type {Sequencer}
     * @private
     */
    this.sequencer = new Sequencer();

    /**
     * Options specifying further behavior of the Datastore
     * @type {Object}
     */
    this.options = mergeDefault(defaultOptions, options);

    // Hash the key if encryption is enabled, otherwise make it null
    this.options.key = this.options.encrypt ? hashKey(this.options.key) : null;

    // Disable spacing if encryption is enabled
    this.options.spacing = this.options.encrypt ? false : this.options.spacing;
    
    // Call initialize if no file exists at the given path
    if (!fs.existsSync(path)) this.init(this.options.data);
  }

  /**
   * Whether the Datastore is finished with file operations currently.
   * @type {Boolean}
   */
  get done() {
    return !this.sequencer.items.length;
  }

  /**
   * Used internally to initialize files.
   */
  init(data) {
    var path = this.path;
    var sequencer = this.sequencer;
    var options = this.options;
    return new Promise(function (resolve, reject) {
      // Add an operation to this Datastore's sequencer
      sequencer.push(async function () {
        await writeJSONFile(path, data, options);
        resolve();
      });
    });
  }

  /**
   * Sets data in the Datastore.
   * @param {Identifier} identifier A string designating the path through an object to a property. `'*'` or an empty string designates the object itself
   * @param {*} val The value to set at the designated property
   * @returns {Promise<Any>} The data resulting from the transformation
   */
  set(identifier, val) {
    var path = this.path;
    var sequencer = this.sequencer;
    var options = this.options;
    return new Promise(function (resolve, reject) {
      // Add an operation to this Datastore's sequencer
      sequencer.push(async function () {
        // Await file edit
        var out;
        await editJSONFile(path, function (data) {
          out = setPropertyInTree(identifier, data, val);
          return out;
        }, options).catch(reject);
        resolve(out);
      });
    });
  }

  /**
   * Gets data from the Datastore.
   * @param {Identifier} identifier A string designating the path through an object to a property. `'*'` or an empty string designates the object itself
   * @returns {Promise<Any>} The data at the given identifier
   */
  get(identifier) {
    var path = this.path;
    var sequencer = this.sequencer;
    var options = this.options;
    return new Promise(function (resolve, reject) {
      sequencer.push(async function () {
        var data = await readJSONFile(path, options).catch(reject);
        resolve(getPropertyInTree(identifier, data));
      });
    });
  }

  /**
   * Transforms the Datastore's data with a given function.
   * @param {Function} fn The given function, taking one argument, `data` as the parsed JSON from the file
   * @returns {Promise<Any>} The data resulting from the transformation
   */
  transform(fn) {
    var path = this.path;
    var sequencer = this.sequencer;
    var options = this.options;
    return new Promise(function (resolve, reject) {
      sequencer.push(async function () {
        var data = await editJSONFile(path, fn, options).catch(reject);
        resolve(data);
      });
    });
  }
}

/**
 * A string designating the path through an object to a property. `'*'` or an empty string designates the object itself
 * @typedef {String} Identifier
 */

module.exports = Datastore;

function hashKey(plaintext) {
  var hash = crypto.createHash('sha256').update(plaintext);
  return Buffer.from(hash.digest());
}

function getIV(key) {
  return crypto.pbkdf2Sync(key, salt, 100, 16, 'sha256');
}

function encrypt(data, key) {
  let iv = getIV(key);
  let cipher = crypto.createCipheriv('aes-256-cbc', Buffer.from(key), iv);
  let encrypted = Buffer.concat([cipher.update(data), cipher.final()]);
  return encrypted.toString('hex');
}

function decrypt(data, key) {
  let iv = getIV(key);
  let encryptedData = Buffer.from(data, 'hex');
  let decipher = crypto.createDecipheriv('aes-256-cbc', Buffer.from(key), iv);
  let decrypted = Buffer.concat([decipher.update(encryptedData), decipher.final()]);
  return decrypted.toString();
}

function mergeDefault(def, given) {
  if (!given) return def;
  for (const key in def) {
    if (!{}.hasOwnProperty.call(given, key)) {
      given[key] = def[key];
    } else if (given[key] === Object(given[key])) {
      given[key] = mergeDefault(def[key], given[key]);
    }
  }

  return given;
}

function editJSONFile(path, func, options) {
  var spaces = options.spacing && !options.encrypt ? 2 : 0;
  return new Promise(function (resolve, reject) {
    fs.readFile(path, 'utf8', function (err, data) {
      if (err) {
        reject(err);
      } else {
        var parsed = JSON.parse(options.encrypt ? decrypt(data, options.key) : data);
        var editedData = JSON.stringify(func(parsed), null, spaces);
        if (options.encrypt) editedData = encrypt(editedData, options.key);
        fs.writeFile(path, editedData, 'utf8', function (err2) {
          if (err2) {
            reject(err2);
          } else {
            resolve();
          }
        });
      }
    });
  });
}

function readJSONFile(path, options) {
  return new Promise(function (resolve, reject) {
    fs.readFile(path, 'utf8', function (err, data) {
      if (err) {
        reject(err);
      } else {
        var parsed = JSON.parse(options.encrypt ? decrypt(data, options.key) : data);
        resolve(parsed);
      }
    });
  });
}

function writeJSONFile(path, data, options) {
  var spaces = options.spacing && !options.encrypt ? 2 : 0;
  return new Promise(function (resolve, reject) {
    var dataToWrite = JSON.stringify(data, null, spaces);
    if (options.encrypt) dataToWrite = encrypt(dataToWrite, options.key);
    fs.writeFile(path, dataToWrite, 'utf8', function (err) {
      if (err) {
        reject(err);
      } else {
        resolve();
      }
    });
  });
}

function getPropertyInTree(identifier, obj) {
  var all = ['*', ''].includes(identifier.trim()) || !identifier;
  if (all) {
    return obj;
  } else {
    // Remove leading periods
    identifier = identifier.match(/[^.].+/)[0];
    var steps = identifier.split('.').filter(e => e.trim().length);
    var current = Object.assign(obj);
    for (let step of steps) {
      // Current step is not an object, cannot proceed
      if (!{}.hasOwnProperty.call(current, step)) {
        throw new Error('Unable to resolve identifier');
      }
      if (step === steps[steps.length - 1]) {
        // Return value if on last step
        return current[step];
      } else {
        // Proceed through path otherwise
        current = current[step];
      }
    }
  }
}

function setPropertyInTree(identifier, obj, val) {
  var all = ['*', ''].includes(identifier.trim())  || !identifier;
  if (all) {
    return val;
  } else {
    // Remove leading periods
    identifier = identifier.match(/[^.].+/)[0];
    var steps = identifier.split('.').filter(e => e.trim().length);
    var current = Object.assign(obj);
    for (let step of steps) {
      if (step === steps[steps.length - 1]) {
        // Set value if on last step of the path
        current[step] = val;
      } else {
        // Current step is not an object, cannot proceed
        if (!{}.hasOwnProperty.call(current, step)) {
          throw new Error('Unable to resolve identifier');
        }
        // Proceed through path otherwise
        current = current[step];
      }
    }
    return obj;
  }
}