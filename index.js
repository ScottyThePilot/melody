const { fork } = require('child_process');
const fs = require('fs');

function log(text) {
  const date = '[' + (new Date()).toISOString().replace(/T/, '][').replace(/Z/, ']');
  console.log(`${date}: ${text}`);
}

function launch() {
  log('Launching Bot...');

  const subprocess = fork('./core/bot.js');

  //subprocess.on('message', (message) => {});

  subprocess.on('exit', (code) => {
    log('Child Exiting with Code: ' + code);
    if (code === 0) {
      log('Relaunching in 10 Seconds...');
      setTimeout(launch, 10000);
    }
  });
}

launch();