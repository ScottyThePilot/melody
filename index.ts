import { fork } from 'child_process';
import { makeLogEntry } from './src/utils/text';

function launch() {
  log('PARENT', 'Launching Application...');

  const subprocess = fork('./src/melody.js');

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

function log(header: string, text?: string, ...rest: string[]) {
  console.log(makeLogEntry(header, text, ...rest));
}
