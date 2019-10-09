'use strict';
const Datastore = require('../modules/Datastore.js');

const blacklist = new Datastore('./core/data/blacklist.json', {
  defaultData: [],
  persistence: true
});

async function blacklistAdd(user) {
  let out;
  await blacklist.edit((data) => {
    if (!data.includes(user.id)) {
      data.push(user.id);
      out = true;
    } else {
      out = false;
    }
  });
  return out;
}

async function blacklistRemove(user) {
  let out;
  await blacklist.edit((data) => {
    if (data.includes(user.id)) {
      data.splice(data.indexOf(user.id), 1);
      out = true;
    } else {
      out = false;
    }
  });
  return out;
}

module.exports = {
  db: blacklist,
  add: blacklistAdd,
  remove: blacklistRemove
};
