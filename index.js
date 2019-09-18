const { fork } = require('child_process');
const Logger = require('./core/modules/Logger.js');
const startTime = new Date();

function log(header, text = '', ...rest) {
  console.log(Logger.makeLogEntry(header, text, ...rest));
}

function launch() {
  log('PARENT', 'Launching Bot...');

  const subprocess = fork('./core/bot.js');

  subprocess.on('message', (message) => {
    if (message.type === 'request') {
      // Child inquires about the lifetime of the parent process
      if (message.head === 'info.lifetime') {
        const lifetime = new Date() - startTime;
        subprocess.send({
          head: 'info.lifetime',
          type: 'response',
          data: lifetime
        });
      }
    }
  });

  subprocess.on('exit', (code) => {
    log('PARENT', 'Child Exiting with Code: ' + code);
    if (code === 0) {
      log('PARENT', 'Relaunching in 10 Seconds...');
      setTimeout(launch, 10000);
    }
  });
}

launch();
