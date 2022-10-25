use linefeed::{Interface, DefaultTerminal, ReadResult, Signal};

use std::thread::{self, JoinHandle};
use std::time::Duration;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Sender, Receiver};

pub fn run(
  body: impl FnOnce(Flag) + Send + 'static,
  terminate: impl FnOnce(Flag) + Send + 'static,
  input: impl Fn(String) + Send + 'static
) {
  let kill_flag = Flag::new();
  let (sender, receiver) = channel();
  setup_logger(sender).unwrap();

  // Body function is spawned in another thread while
  // the terminal runs on this thread
  let body_handle = {
    let kill_flag = kill_flag.clone();
    thread::spawn(move || body(kill_flag))
  };

  run_terminal(receiver, input, terminate, kill_flag, &body_handle);
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

fn run_terminal(
  receiver: Receiver<String>,
  input: impl Fn(String) + Send + 'static,
  terminate: impl FnOnce(Flag) + Send + 'static,
  kill_flag: Flag,
  body_handle: &JoinHandle<()>
) {
  let interface = Interface::new(env!("CARGO_PKG_NAME")).unwrap();
  interface.set_prompt("> ").unwrap();
  interface.set_report_signal(Signal::Interrupt, true);

  let timeout = Duration::from_secs_f32(1.0 / 30.0);

  fn pipe_lines(interface: &Interface<DefaultTerminal>, receiver: &Receiver<String>) {
    for line in receiver.try_iter() {
      write!(interface, "{}", line).unwrap();
    };
  }

  let mut terminate = Some(terminate);

  loop {
    if kill_flag.get() { break };
    if body_handle.is_finished() { return };
    match interface.read_line_step(Some(timeout)).unwrap() {
      Some(ReadResult::Eof) | None => (),
      Some(ReadResult::Input(line)) => input(line),
      Some(ReadResult::Signal(signal)) => match signal {
        Signal::Interrupt => terminate_handler(&kill_flag, &mut terminate),
        signal => println!("Signal: {signal:?}")
      }
    };

    pipe_lines(&interface, &receiver);
  };

  while !body_handle.is_finished() {
    pipe_lines(&interface, &receiver);
  };
}

fn terminate_handler(kill_flag: &Flag, terminate: &mut Option<impl FnOnce(Flag)>) {
  if let Some(terminate) = terminate.take() {
    terminate(kill_flag.clone())
  } else {
    kill_flag.set();
  };
}

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct Flag(Arc<AtomicBool>);

impl Flag {
  pub fn new() -> Self {
    Flag(Arc::new(AtomicBool::new(false)))
  }

  pub fn get(&self) -> bool {
    self.0.load(Ordering::Relaxed)
  }

  pub fn set(&self) {
    self.0.store(true, Ordering::Relaxed);
  }
}
