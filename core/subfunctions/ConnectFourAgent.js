'use strict';
const Lazystore = require('../structures/Lazystore.js');
const Queue = require('../structures/Queue.js');
const ConnectFourGame = require('./ConnectFourGame.js');

class ConnectFourAgent {
  constructor(location) {
    this.lb = new Lazystore(location, {
      defaultData: []
    });
    this.queue = new Queue();
    
    this.queue.push(() => {
      return this.lb.init().then(() => {
        for (let i = 0; i < this.lb.state.length; i ++)
          this.lb.state[i] = ConnectFourGame.from(this.lb.state[i]);
      });
    });
  }

  createGame(playerA, playerB) {
    this.lb.state.push(new ConnectFourGame(playerA, playerB));
    this.lb.touch();
  }

  write() {
    return this.queue.pushPromise(() => this.lb.write());
  }

  find(player) {
    for (let game of this.lb.state)
      if (game.hasPlayer(player)) return game;
    return null;
  }
}

module.exports = ConnectFourAgent;
