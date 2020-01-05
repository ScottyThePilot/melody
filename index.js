'use strict';
const { fork } = require('child_process');
const { logEntryToConsole: log } = require('./core/modules/util.js');
const Communicator = require('./core/structures/Communicator.js');

const startTime = new Date();

function launch() {
  log('PARENT', 'Launching Bot...');

  const subprocess = fork('./core/melody.js');
  const comm = new Communicator(subprocess);

  comm.on('info.lifetime', (message) => {
    if (message.type !== 'request') return;
    const data = new Date() - startTime;
    comm.send('info.lifetime', {
      type: 'response',
      data
    });
  });

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
