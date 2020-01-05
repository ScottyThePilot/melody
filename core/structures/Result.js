'use strict';

class Result {
  get ok() {
    return this instanceof Result.Ok;
  }

  get err() {
    return this instanceof Result.Err;
  }

  map(fn) {
    if (this.ok) return fn(this.value);
    return this;
  }

  unwrapElse(def) {
    if (this.ok) return this.value;
    return def;
  }

  unwrap(error) {
    if (this.ok) return this.value;
    throw new Error(this.error || error);
  }
}

Result.Ok = class Ok extends Result {
  constructor(value) {
    super();
    this.value = value;
  }

  valueOf() {
    return this.value;
  }
};

Result.Err = class Err extends Result {
  constructor(error) {
    super();
    this.error = error;
  }

  valueOf() {
    return this.error;
  }
};

module.exports = Result;
