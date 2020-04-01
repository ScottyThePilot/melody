import Group from './Group';

export default class Table<K, V> extends Map<K, V> {
  /**
   * Returns true if every entry passes the check specified by
   * the `fn` parameter, and false otherwise.
   */
  every(fn: (value: V, key: K, table: this) => boolean): boolean {
    for (const [k, v] of this) if (!fn(v, k, this)) return false;
    return true;
  }

  /**
   * Returns true if at least one entry passes the check specified
   * by the `fn` parameter, and false otherwise.
   */
  some(fn: (value: V, key: K, table: this) => boolean): boolean {
    for (const [k, v] of this) if (fn(v, k, this)) return true;
    return false;
  }

  /**
   * Returns a new Table containing only elements that passed the
   * check specified by the `fn` parameter.
   */
  filter(fn: (value: V, key: K, table: this) => boolean): Table<K, V> {
    let out = new Table<K, V>();
    for (let [k, v] of this) if (fn(v, k, this)) out.set(k, v);
    return out;
  }

  /**
   * Returns the first entry in the Table that passes the check
   * specified by the `fn` parameter. If no entries pass the check,
   * undefined is returned.
   */
  find(fn: (value: V, key: K, table: this) => boolean): [K, V] | undefined {
    for (const [k, v] of this) if (fn(v, k, this)) return [k, v];
    return;
  }
  
  /**
   * Creates a new Table with the results of calling the provided
   * function on every value.
   */
  map<U>(fn: (value: V, key: K, table: this) => U): Table<K, U> {
    let out = new Table<K, U>();
    for (const [k, v] of this) out.set(k, fn(v, k, this));
    return out;
  }

  /**
   * Creates a new Group with all of this Table's values.
   */
  valuesGroup(): Group<V> {
    return new Group<V>(this.values());
  }

  /**
   * Creates a new Group will all of this Table's keys.
   */
  keysGroup(): Group<K> {
    return new Group<K>(this.keys());
  }
}
