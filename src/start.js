"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const Melody_1 = __importDefault(require("@core/Melody"));
const config = require('./config.json');
process.on('unhandledRejection', (reason) => { throw reason; });
Melody_1.default.create(config).then((melody) => {
    melody.on('command', onCommand.bind(melody));
});
function onCommand(command) {
}
