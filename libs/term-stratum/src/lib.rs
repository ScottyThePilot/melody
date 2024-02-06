use linefeed::{Interface, ReadResult, Signal};

use std::thread::{self, JoinHandle};
use std::time::Duration;
use std::sync::mpsc::Receiver;

/// This function spawns the `program_body` in another thread, waiting for it to finish.
/// For the duration of the `program_body`, a readline will be opened in the terminal for accepting input.
/// This input will be passed to `event_handler`. If the user enters Ctrl-C, the readline stops accepting input,
/// and a `StratumEvent::Terminate` is sent. A reciever channel must also be provided,
/// which, for the duration of this function, is the only valid way to print output to the terminal.
pub fn run(
  name: &'static str,
  logger_reciever: Receiver<String>,
  program_body: impl FnOnce() + Send + 'static,
  event_handler: impl Fn(StratumEvent) + Send + 'static
) {
  let body_handle = thread::spawn(program_body);
  run_terminal(name, logger_reciever, event_handler, &body_handle);
  body_handle.join().unwrap();
  println!();
}

fn run_terminal(
  name: &'static str,
  logger_reciever: Receiver<String>,
  event_handler: impl Fn(StratumEvent) + Send + 'static,
  body_handle: &JoinHandle<()>
) {
  let interface = Interface::new(name).unwrap();
  interface.set_prompt("> ").unwrap();
  interface.set_report_signal(Signal::Interrupt, true);

  let timeout = Duration::from_secs_f32(1.0 / 30.0);

  loop {
    if body_handle.is_finished() { return };
    match interface.read_line_step(Some(timeout)).unwrap() {
      Some(ReadResult::Eof) | None => (),
      Some(ReadResult::Input(line)) => event_handler(StratumEvent::Input(line)),
      Some(ReadResult::Signal(Signal::Interrupt)) => {
        event_handler(StratumEvent::Terminate);
        break;
      },
      Some(ReadResult::Signal(..)) => ()
    };

    for line in logger_reciever.try_iter() {
      write!(interface, "{}", line).unwrap();
    };
  };

  for line in logger_reciever.iter() {
    write!(interface, "{}", line).unwrap();
  };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StratumEvent {
  Input(String),
  Terminate
}
