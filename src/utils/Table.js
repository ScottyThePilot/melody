'use strict';

/**
 * @template K, V
 * @extends {Map<K, V>}
 */
export default class Table extends Map {
  /**
   * Returns true if every entry passes the check specified by
   * the `fn` parameter, and false otherwise.
   * @param {(value: V, key: K, table: this) => boolean} fn
   * @returns {boolean}
   */
  every(fn) {
    for (const [k, v] of this) if (!fn(v, k, this)) return false;
    return true;
  }

  /**
   * Returns true if at least one entry passes the check specified
   * by the `fn` parameter, and false otherwise.
   * @param {(value: V, key: K, table: this) => boolean} fn
   * @returns {boolean}
   */
  some(fn) {
    for (const [k, v] of this) if (fn(v, k, this)) return true;
    return false;
  }

  /**
   * Returns a new Table containing only elements that passed the
   * check specified by the `fn` parameter.
   * @param {(value: V, key: K, table: this) => boolean} fn
   * @returns {Table<K, V>}
   */
  filter(fn) {
    let out = new Table();
    for (let [k, v] of this) if (fn(v, k, this)) out.set(k, v);
    return out;
  }

  /**
   * Returns the first entry in the Table that passes the check
   * specified by the `fn` parameter. If no entries pass the check,
   * undefined is returned.
   * @param {(value: V, key: K, table: this) => boolean} fn
   * @returns {[K, V] | undefined}
   */
  find(fn) {
    for (const [k, v] of this) if (fn(v, k, this)) return [k, v];
    return;
  }
  
  /**
   * Creates a new Table with the results of calling the provided
   * function on every value.
   * @param {(value: V, key: K, table: this) => U} fn
   * @returns {Table<K, U>}
   */
  map(fn) {
    let out = new Table();
    for (const [k, v] of this) out.set(k, fn(v, k, this));
    return out;
  }
}
