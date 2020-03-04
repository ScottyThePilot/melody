'use strict';

/** @template T */
class Collection extends Set {
  /**
   * Returns an array containing the same elements as this Collection.
   * @returns {Array<T>}
   */
  array() {
    return Array.from(this);
  }
  
  /**
   * Returns true if every element passes the check specified by
   * the `fn` parameter, and false otherwise.
   * @param {(el: T, coll: this) => boolean} fn
   * @returns {boolean}
   */
  every(fn) {
    for (let e of this) if (!fn(e, this)) return false;
    return true;
  }

  /**
   * Returns true if at least one element passes the check specified
   * by the `fn` parameter, and false otherwise.
   * @param {(el: T, coll: this) => boolean} fn
   * @returns {boolean}
   */
  some(fn) {
    for (let e of this) if (fn(e, this)) return true;
    return false;
  }

  /**
   * Returns a new Collection containing only elements that passed the
   * check specified by the `fn` parameter.
   * @param {(el: T, coll: this) => boolean} fn
   * @returns {Collection<T>}
   */
  filter(fn) {
    let out = new Collection();
    for (let e of this) if (fn(e, this)) out.add(e);
    return out;
  }

  /**
   * Returns the first element in the Collection that passes the check
   * specified by the `fn` parameter. If no elements pass the check,
   * undefined is returned.
   * @param {(el: T, coll: this) => boolean} fn 
   * @returns {T}
   */
  find(fn) {
    for (let e of this) if (fn(e, this)) return e;
  }
  
  /**
   * Creates a new Collection with the results of calling the provided
   * function on every element.
   * @template N
   * @param {(el: T, coll: this) => N} fn 
   * @returns {Collection<N>}
   */
  map(fn) {
    let out = new Collection();
    for (let e of this) out.add(fn(e, this));
    return out;
  }
}

module.exports = Collection;
