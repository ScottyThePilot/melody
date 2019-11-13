'use strict';
const Lazystore = require('../modules/Lazystore.js');
const ConnectFourGame = require('./ConnectFourGame.js');

class ConnectFourAgent extends Lazystore {
  constructor(location) {
    super(location, {
      defaultData: []
    });
  }

  

  async init() {
    await super.init();
    for (let i = 0; i < this.state.length; i ++)
      this.state[i] = ConnectFourGame.from(this.state[i]);
  }
}

ConnectFourAgent.prototype.get = undefined;
ConnectFourAgent.prototype.set = undefined;
ConnectFourAgent.prototype.has = undefined;

module.exports = ConnectFourAgent;

/*
user1 challenges user2
  if user2 has a pending challenge for user1
    create a game between user1 and user2
    user2 goes first
  else
    add a pending challenge
*/
