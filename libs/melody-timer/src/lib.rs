use chrono::{DateTime, Utc};
use tokio::sync::mpsc::{UnboundedSender, UnboundedReceiver, unbounded_channel};
use tokio::sync::mpsc::error::SendError;
use tokio::time::{Duration, Instant, sleep_until};

use std::collections::BinaryHeap;
use std::cmp::Ordering;



pub fn timer<T, D: Deadline>() -> (TimerSender<T, D>, TimerReceiver<T, D>) {
  let (sender, reciever) = unbounded_channel();
  (TimerSender { sender }, TimerReceiver { reciever, deadlines: BinaryHeap::new() })
}

pub type TimerSenderInstant<T> = TimerSender<T, Instant>;
pub type TimerReceiverInstant<T> = TimerReceiver<T, Instant>;
pub type TimerSenderDateTime<T> = TimerSender<T, DateTime<Utc>>;
pub type TimerReceiverDateTime<T> = TimerReceiver<T, DateTime<Utc>>;

#[derive(Debug)]
pub struct TimerSender<T, D: Deadline> {
  sender: UnboundedSender<TimerItem<T, D>>
}

impl<T, D: Deadline> TimerSender<T, D> {
  pub fn send(&self, value: T, deadline: D) -> Result<(), T> {
    self.sender.send(TimerItem { deadline, value }).map_err(|SendError(e)| e.value)
  }
}

impl<T, D: Deadline> Clone for TimerSender<T, D> {
  fn clone(&self) -> Self {
    TimerSender { sender: self.sender.clone() }
  }
}

#[derive(Debug)]
pub struct TimerReceiver<T, D: Deadline> {
  reciever: UnboundedReceiver<TimerItem<T, D>>,
  deadlines: BinaryHeap<TimerItem<T, D>>
}

impl<T, D: Deadline> TimerReceiver<T, D> {
  fn next_deadline(&self) -> Option<Instant> {
    self.deadlines.peek().map(|item| item.deadline.to_instant())
  }

  pub async fn next(&mut self) -> Option<T> {
    loop {
      let result = match self.next_deadline() {
        None => Err(self.deadlines.push(self.reciever.recv().await?)),
        Some(instant) => tokio::select! {
          item = self.reciever.recv() => Err(self.deadlines.push(item?)),
          () = sleep_until(instant) => Ok(self.deadlines.pop().unwrap())
        }
      };

      match result {
        Ok(TimerItem { value, .. }) => break Some(value),
        Err(()) => continue
      };
    }
  }

  pub fn push(&mut self, value: T, deadline: D) {
    self.deadlines.push(TimerItem { deadline, value });
  }
}

#[derive(Debug)]
struct TimerItem<T, D: Deadline> {
  deadline: D,
  value: T
}

impl<T, D: Deadline> Eq for TimerItem<T, D> {}

impl<T, D: Deadline> PartialEq for TimerItem<T, D> {
  fn eq(&self, other: &Self) -> bool {
    self.deadline == other.deadline
  }
}

impl<T, D: Deadline> Ord for TimerItem<T, D> {
  fn cmp(&self, other: &Self) -> Ordering {
    self.deadline.cmp(&other.deadline).reverse()
  }
}

impl<T, D: Deadline> PartialOrd for TimerItem<T, D> {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

pub trait Deadline: Copy + Ord + Eq {
  fn to_instant(self) -> Instant;
}

impl Deadline for Instant {
  fn to_instant(self) -> Instant {
    self
  }
}

impl Deadline for DateTime<Utc> {
  fn to_instant(self) -> Instant {
    let now_date_time = Utc::now();
    let now_instant = Instant::now();
    let duration = self
      .signed_duration_since(now_date_time)
      .to_std().ok().unwrap_or(Duration::ZERO);
    now_instant + duration
  }
}
