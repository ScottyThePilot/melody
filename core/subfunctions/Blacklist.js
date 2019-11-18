'use strict';
const Datastore = require('../structures/Datastore.js');

class Blacklist {
  constructor(location) {
    this.db = new Datastore(location, {
      defaultData: [],
      persistence: true
    });
  }

  async add(user) {
    const id = user ? user.id : user;
    let out;
    await this.db.edit((data) => {
      if (!data.includes(id)) {
        data.push(id);
        out = true;
      } else {
        out = false;
      }
    });
    return out;
  }

  async remove(user) {
    const id = user ? user.id : user;
    let out;
    await this.db.edit((data) => {
      if (data.includes(id)) {
        data.splice(data.indexOf(id), 1);
        out = true;
      } else {
        out = false;
      }
    });
    return out;
  }
}

module.exports = Blacklist;
