'use strict';
const Result = require('../structures/Result.js');

class ConnectFourGame {
  constructor(playerA, playerB, state = fill(42), turn = 2, lastActive = new Date()) {
    this.playerA = playerA;
    this.playerB = playerB;
    this.state = state;
    this.turn = turn;
    this.lastActive = lastActive;
  }

  hasPlayer(player) {
    return this.playerA === player || this.playerB === player;
  }

  get activeTime() {
    return new Date() - this.lastActive;
  }

  checkWinner() {
    return checkVertical(this.state)
      || checkHorizontal(this.state)
      || checkDiagonal1(this.state)
      || checkDiagonal2(this.state);
  }

  isColumnFull(column) {
    return !this.state[column];
  }

  placeChecker(column, player) {
    for (let y = 5; y >= 0; y --) {
      const l = y * 7 + column;
      if (!this.state[l]) {
        this.state[l] = player;
        break;
      }
    }
  }

  play(column, player) {
    if (!this.canPlay(column))
      return new Result.Err();
    this.placeChecker(column, player);
    this.turn = -this.turn + 3;
    return this.checkWinning()
      ? new Result.Ok(player)
      : new Result.Ok();
  }

  toJSON() {
    const state = encode(this.state.join(''));
    return { ...this, state };
  }

  static from(raw) {
    const state = Array.from(decode(raw.state)).map((e) => +e);
    const lastActive = new Date(raw.lastActive);
    return createObject(ConnectFourGame, { ...raw, state, lastActive });
  }
}

module.exports = ConnectFourGame;

function createObject(obj, props) {
  return Object.assign(Object.create(obj.prototype), props);
}

function encode(str) {
  return Buffer.from(str).toString('base64');
}

function decode(str) {
  return Buffer.from(str, 'base64').toString();
}

function fill(len) {
  let out = [];
  for (let i = 0; i < len; i ++) out[i] = 0;
  return out;
}

function checkVertical(state) {
  // Scan through each column
  for (let x = 0; x < 7; x ++) {
    let s1 = 0; // Successive player 1 checkers
    let s2 = 0; // Successive player 2 checkers
    // Scan through each checker in the column
    for (let y = 0; y < 6; y ++) {
      // Increment/reset counts based on checker in column
      switch (state[y * 7 + x]) {
        case 0: s1 = 0; s2 = 0; break;
        case 1: s1 ++;  s2 = 0; break;
        case 2: s1 = 0; s2 ++;  break;
      }
      // If there are 4 of any checker lined up, return
      if (s1 >= 4) return 1;
      if (s2 >= 4) return 2;
    }
  }
  // Return 0 if there are none lined up
  return 0;
}

function checkHorizontal(state) {
  // Scan through each row
  for (let y = 0; y < 6; y ++) {
    let s1 = 0; // Successive player 1 checkers
    let s2 = 0; // Successive player 2 checkers
    // Scan through each checker in the row
    for (let x = 0; x < 7; x ++) {
      // Increment/reset counts based on checker in row
      switch (state[y * 7 + x]) {
        case 0: s1 = 0; s2 = 0; break;
        case 1: s1 ++;  s2 = 0; break;
        case 2: s1 = 0; s2 ++;  break;
      }
      // If there are 4 of any checker lined up, return
      if (s1 >= 4) return 1;
      if (s2 >= 4) return 2;
    }
  }
  // Return 0 if there are none lined up
  return 0;
}

function checkDiagonal1(state) {
  // Scan through each diagonal
  for (let i = 3; i < 9; i ++) {
    const t = Math.min(i + 1, 12 - i);
    let s1 = 0; // Successive player 1 checkers
    let s2 = 0; // Successive player 2 checkers
    // Scan through each checker in the diagonal
    for (let j = 0; j < t; j ++) {
      const l = (i > 6 ? i * 7 - 36 : i) + j * 6;
      // Increment/reset counts based on checker in column
      switch (state[l]) {
        case 0: s1 = 0; s2 = 0; break;
        case 1: s1 ++;  s2 = 0; break;
        case 2: s1 = 0; s2 ++;  break;
      }
      // If there are 4 of any checker lined up, return
      if (s1 >= 4) return 1;
      if (s2 >= 4) return 2;
    }
  }
  return 0;
}

function checkDiagonal2(state) {
  // Scan through each diagonal
  for (let i = -5; i < 7; i ++) {
    const t = Math.min(i + 6, 7 - i);
    let s1 = 0; // Successive player 1 checkers
    let s2 = 0; // Successive player 2 checkers
    // Scan through each checker in the diagonal
    for (let j = 0; j < t; j ++) {
      const l = (i < 0 ? i * -7 : i) + j * 8;
      // Increment/reset counts based on checker in column
      switch (state[l]) {
        case 0: s1 = 0; s2 = 0; break;
        case 1: s1 ++;  s2 = 0; break;
        case 2: s1 = 0; s2 ++;  break;
      }
      // If there are 4 of any checker lined up, return
      if (s1 >= 4) return 1;
      if (s2 >= 4) return 2;
    }
  }
  return 0;
}
