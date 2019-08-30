'use strict';

class Util {
  constructor() {
    throw new Error(`The ${this.constructor.name} class may not be instantiated`);
  }

  static shuffle(array) {
    if (!Array.isArray(array)) throw new TypeError('Expected an array');
    var arr = array.slice(0);
    var currentIndex = arr.length, temporaryValue, randomIndex;

    while (0 !== currentIndex) {
      randomIndex = Math.floor(Math.random() * currentIndex);
      currentIndex -= 1;

      temporaryValue = arr[currentIndex];
      arr[currentIndex] = arr[randomIndex];
      arr[randomIndex] = temporaryValue;
    }

    return arr;
  } // Functional

  /**
   * Sets default properties on an object that aren't already specified
   * @param {Object} def Default properties
   */
  static mergeDefault(def, given) {
    if (!given) return def;
    for (const key in def) {
      if (!{}.hasOwnProperty.call(given, key)) {
        given[key] = def[key];
      } else if (given[key] === Object(given[key])) {
        given[key] = Util.mergeDefault(def[key], given[key]);
      }
    }

    return given;
  } // Functional

  static intersectDefault(def, given) {
    if (!given) return def;
    for (const key in given) {
      if (!{}.hasOwnProperty.call(def, key)) {
        delete given[key];
      }
    }

    return given;
  } // Functional

  /**
   * Generates a random, non-unique ID consisting of numbers and lowercase letters
   * @param {Number} len The desired length of the ID
   */
  static generateID(len = 8) {
    if (!Util.isPosInt(len)) throw new RangeError('Invalid id length');
    return Array(Math.ceil(len / 8)).fill(void 0).map(e => Math.random().toString(36).slice(2, 10)).join('').slice(0, len);
  } // Functional

  /**
   * Replaces markers in a String with given values
   * @param {String} str The String to have replacements applied
   * @param {...String} replacers Strings to insert
   * @returns {String}
   */
  static format(str, ...replacers) {
    if (typeof str !== 'string') throw new TypeError('Expected a string');
    return str.replace(/{(\d+)}/g, function(match, number) {
      return typeof replacers[number] !== 'undefined' ? replacers[number] : match;
    });
  }

  /**
   * Tests if a value can be used arithmetally, and whether it is equal to or between 1 and 0
   * @param {*} val The value to be tested
   * @returns {Boolean}
   */
  static isPercent(val) {
    !isNaN(val) && val >= 0 && val <= 1;
  } // Tester

  /**
   * Tests if a value is an integer and greater than or equal to zero
   * @param {*} val The value to be tested
   * @returns {Boolean}
   */
  static isPosInt(val) {
    return parseInt(val) !== Infinity && Number.isInteger(val) && val >= 0;
  } // Tester

  /**
   * Tests if a value is Array-like
   * @param {*} val The value to be tested
   * @returns {Boolean}
   */
  static isArrayLike(val) {
    return val !== null && typeof val[Symbol.iterator] === 'function' && typeof val.length === 'number';
  } // Tester

  /**
   * A stricter, inverse version of `isNaN`
   * @param {*} val The value to be tested
   * @returns {Boolean}
   */
  static isNumeric(val) {
    return !isNaN(parseFloat(val)) && isFinite(val);
  } // Tester

  /**
   * Picks a random value from an Array
   * @param {Array} arr The Array to be picked from
   */
  static pick(arr) {
    if (!Util.isArrayLike(arr)) throw new TypeError('Expected an array-like object');
    return arr.length === 1 ? arr[0] : arr[Math.floor(Math.random() * arr.length)];
  }

  static capFirst(str) {
    return ''.charAt.call(str, 0).toUpperCase() + ''.slice.call(str, 1);
  }

  static random(v0, v1) {
    return Util.lerp(v0, v1, Math.random());
  } // Functional

  static irandom(v0, v1) {
    return Math.floor(Util.random(v0, v1));
  }

  static lerp(v0, v1, t) {
    return v0 * (1 - t) + v1 * t;
  } // Functional

  static clamp(val, max, min) {
    return Math.min(Math.max(val, min), max);
  }

  static modulo(val, max) {
    return (val % max < 0 ? max : 0) + (val % max);
  }

  static async asyncForEach(array, callback) {
    for (let index = 0; index < array.length; index++) {
      await callback(array[index], index, array);
    }
  }

  static dround(val, d = 0) {
    return ({[-1]: Math.floor, [0]: Math.round, [1]: Math.ceil})[Math.sign(d)](val);
  }

  static dist(x1, y1, x2, y2) {
    var h = x1 - x2, k = y1 - y2;
    return Math.sqrt(h*h + k*k);
  }
}

module.exports = Util;
