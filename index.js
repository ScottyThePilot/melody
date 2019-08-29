const { fork } = require('child_process');
const fs = require('fs');

const errLogStream = fs.createWriteStream('/err.log', { flags: 'a' });

function log(text) {
  const date = '[' + (new Date()).toISOString().replace(/T/, '][').replace(/Z/, ']');
  console.log(`${date}: ${text}`);
}

function logErr(err) {
  errLogStream.write(err);
}

function launch() {
  log('Launching Bot...');

  const subprocess = fork('./core/bot.js');

  subprocess.on('message', (message) => {
    if (message.err) logErr(message.err);
  });

  subprocess.on('exit', (code) => {
    log('Child Exiting with Code: ' + code);
    if (code === 0) {
      log('Relaunching in 10 Seconds...');
      setTimeout(launch, 10000);
    }
  });
}

launch();