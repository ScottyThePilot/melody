'use strict';
import Melody from './core/Melody.js';
import fs from 'fs';

const config = JSON.parse(fs.readFileSync('./src/config.json'));

process.on('unhandledRejection', (reason) => { throw reason; });

Melody.create(config).then((melody) => {
  process.on('SIGHUP', () => {
    melody.destroy().then(() => process.exit());
  });

  melody.on('command', (...args) => {
    console.log(args.join(' '));
  });
});


