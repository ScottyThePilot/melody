use linefeed::{Interface, DefaultTerminal, ReadResult, Signal};

use std::thread::{self, JoinHandle};
use std::time::Duration;
use std::sync::mpsc::{channel, Sender, Receiver};

pub fn run(body: fn(), input: fn(String)) {
  let (sender, receiver) = channel();
  setup_logger(sender).unwrap();

  // Body function is spawned in another thread while
  // the terminal runs on this thread
  let body_handle = thread::spawn(body);
  run_terminal(receiver, input, &body_handle);
  body_handle.join().unwrap();
}

fn setup_logger(sender: Sender<String>) -> Result<(), fern::InitError> {
  let me = env!("CARGO_PKG_NAME").replace('-', "_");
  fern::Dispatch::new()
    .format(move |out, message, record| {
      out.finish(format_args!(
        "{}[{}]({}) {}",
        chrono::Local::now().format("[%H:%M:%S]"),
        record.level(),
        record.target(),
        message
      ))
    })
    .level(log::LevelFilter::Warn)
    .level_for(me, log::LevelFilter::Trace)
    .chain(sender)
    .chain({
      std::fs::create_dir_all("./data/")?;
      fern::log_file("./data/latest.log")?
    })
    .apply()?;
  Ok(())
}

fn run_terminal(receiver: Receiver<String>, input: fn(String), body_handle: &JoinHandle<()>) {
  let interface = Interface::new(env!("CARGO_PKG_NAME")).unwrap();
  interface.set_prompt("> ").unwrap();
  interface.set_report_signal(Signal::Break, true);
  interface.set_report_signal(Signal::Continue, true);
  interface.set_report_signal(Signal::Interrupt, true);
  interface.set_report_signal(Signal::Suspend, true);
  interface.set_report_signal(Signal::Quit, true);

  let timeout = Duration::from_secs_f32(1.0 / 30.0);

  fn pipe_lines(interface: &Interface<DefaultTerminal>, receiver: &Receiver<String>) {
    for line in receiver.try_iter() {
      write!(interface, "{}", line).unwrap();
    };
  }

  loop {
    if self::interrupt::was_killed() { break };
    if body_handle.is_finished() { return };
    match interface.read_line_step(Some(timeout)).unwrap() {
      Some(ReadResult::Eof) | None => (),
      Some(ReadResult::Input(line)) => input(line),
      Some(ReadResult::Signal(signal)) => match signal {
        Signal::Interrupt => self::interrupt::take_handler()(),
        signal => println!("Signal: {signal:?}")
      }
    };

    pipe_lines(&interface, &receiver);
  };

  while !body_handle.is_finished() {
    pipe_lines(&interface, &receiver);
  };
}

pub mod interrupt {
  use parking_lot::{const_mutex, Mutex};
  use std::sync::atomic::{AtomicBool, Ordering};

  pub type HandlerFunction = Box<dyn FnOnce() + Send + Sync + 'static>;

  static KILL: AtomicBool = AtomicBool::new(false);
  static HANDLER: Mutex<Option<HandlerFunction>> = const_mutex(None);

  pub(super) fn take_handler() -> HandlerFunction {
    HANDLER.lock().take().unwrap_or_else(|| Box::new(kill))
  }

  pub fn reset_handler() {
    *HANDLER.lock() = None;
  }

  pub fn set_handler(handler: impl FnOnce() + Send + Sync + 'static) {
    *HANDLER.lock() = Some(Box::new(handler) as HandlerFunction);
  }

  pub fn kill() {
    KILL.store(true, Ordering::Relaxed);
  }

  pub fn was_killed() -> bool {
    KILL.load(Ordering::Relaxed)
  }
}
