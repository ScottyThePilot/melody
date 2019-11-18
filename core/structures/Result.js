'use strict';

class Result {
  get ok() {
    return this instanceof Result.Ok;
  }

  get err() {
    return this instanceof Result.Err;
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
