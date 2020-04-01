"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const child_process_1 = require("child_process");
const text_1 = require("./src/utils/text");
function launch() {
    log('PARENT', 'Launching Application...');
    const subprocess = child_process_1.fork('./src/melody.js');
    subprocess.on('exit', (code) => {
        log('PARENT', 'Child Exiting with Code: ' + code);
        if (code === 0) {
            log('PARENT', 'Relaunching in 10 Seconds...');
            setTimeout(launch, 10000);
        }
        else {
            process.exit();
        }
    });
}
launch();
function log(header, text, ...rest) {
    console.log(text_1.makeLogEntry(header, text, ...rest));
}
