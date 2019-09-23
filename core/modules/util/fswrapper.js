const fs = require('fs');
const { promisify: p } = require('util');

module.exports = {
  write: p(fs.writeFile),
  read: p(fs.readFile),

  exists: fs.existsSync,
  stat: p(fs.stat),

  mkdir: p(fs.mkdir),
  rmdir: p(fs.rmdir),

  readdir: p(fs.readdir)
};