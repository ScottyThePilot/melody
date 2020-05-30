'use strict';

/**
 * @template T
 * @extends {Set<T>}
 */
export default class Group extends Set {
  /**
   * Returns an array containing the same elements as this Group.
   * @returns {T[]}
   */
  array() {
    return [...this.values()];
  }
  
  /**
   * Returns true if every element passes the check specified by
   * the `fn` parameter, and false otherwise.
   * @param {(element: T, group: this) => boolean} fn
   * @returns {boolean}
   */
  every(fn) {
    for (let e of this) if (!fn(e, this)) return false;
    return true;
  }

  /**
   * Returns true if at least one element passes the check specified
   * by the `fn` parameter, and false otherwise.
   * @param {(element: T, group: this) => boolean} fn
   * @returns {boolean}
   */
  some(fn) {
    for (let e of this) if (fn(e, this)) return true;
    return false;
  }

  /**
   * Returns a new Group containing only elements that passed the
   * check specified by the `fn` parameter.
   * @param {(element: T, group: this) => boolean} fn
   * @returns {Group<T>}
   */
  filter(fn) {
    let out = new Group();
    for (let e of this) if (fn(e, this)) out.add(e);
    return out;
  }

  /**
   * Returns the first element in the Group that passes the check
   * specified by the `fn` parameter. If no elements pass the check,
   * undefined is returned.
   * @param {(element: T, group: this) => boolean} fn
   * @returns {T | undefined}
   */
  find(fn) {
    for (let e of this) if (fn(e, this)) return e;
    return;
  }
  
  /**
   * Creates a new Group with the results of calling the provided
   * function on every element.
   * @template U
   * @param {(element: T, group: this) => U} fn
   * @returns {Group<U>}
   */
  map(fn) {
    let out = new Group();
    for (let e of this) out.add(fn(e, this));
    return out;
  }
}
