'use strict';
import Util from './src/utils/Util.js';
import { fork } from 'child_process';

let exit = false;
let subprocess = null;

process.on('SIGHUP', () => {
  exit = true;
  if (subprocess) {
    log('PARENT', 'Killing Child Process...');
    subprocess.kill('SIGHUP');
    subprocess.removeAllListeners();
    subprocess.once('exit', (code) => {
      log('PARENT', 'Child Killed with Code: ' + code);
      process.exit();
    });
  } else {
    process.exit();
  }
});

function launch() {
  if (exit) return;

  log('PARENT', 'Launching Application...');

  subprocess = fork('./src/start.js');
  subprocess.on('exit', (code) => {
    log('PARENT', 'Child Exiting with Code: ' + code);
    if (code === 0) {
      log('PARENT', 'Relaunching in 10 Seconds...');
      setTimeout(launch, 10000);
    } else {
      process.exit();
    }
  });
}

launch();

function log(header, text, ...rest) {
  console.log(Util.makeLogEntry(header, text, ...rest));
}
