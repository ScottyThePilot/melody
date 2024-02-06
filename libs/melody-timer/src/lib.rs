use chrono::{DateTime, Utc};
use futures::future::Either;
use tokio::sync::mpsc::{Sender, Receiver, channel};
use tokio::sync::mpsc::error::{SendError, TrySendError};
use tokio::time::{Duration, Instant, sleep_until};

use std::collections::BinaryHeap;
use std::cmp::Ordering;
use std::future::pending;



pub fn timer<N>(buffer: usize) -> (TimerSender<N>, TimerReceiver<N>) {
  let (sender, reciever) = channel(buffer);
  (TimerSender { sender }, TimerReceiver { reciever, deadlines: BinaryHeap::new() })
}

#[derive(Debug)]
pub struct TimerSender<N> {
  sender: Sender<TimerItem<N>>
}

impl<N> TimerSender<N> {
  pub async fn send(&self, value: N, deadline: DateTime<Utc>) -> Result<(), N> {
    self.sender.send(TimerItem { deadline, value }).await.map_err(|SendError(e)| e.value)
  }

  pub async fn try_send(&self, value: N, deadline: DateTime<Utc>) -> Result<(), N> {
    self.sender.try_send(TimerItem { deadline, value }).map_err(|e| match e {
      TrySendError::Full(e) | TrySendError::Closed(e) => e.value
    })
  }

  pub fn blocking_send(&self, value: N, deadline: DateTime<Utc>) -> Result<(), N> {
    self.sender.blocking_send(TimerItem { deadline, value }).map_err(|SendError(e)| e.value)
  }
}

impl<N> Clone for TimerSender<N> {
  fn clone(&self) -> Self {
    TimerSender { sender: self.sender.clone() }
  }
}

#[derive(Debug)]
pub struct TimerReceiver<N> {
  reciever: Receiver<TimerItem<N>>,
  deadlines: BinaryHeap<TimerItem<N>>
}

impl<N> TimerReceiver<N> {
  pub fn next_deadline(&self) -> Option<Instant> {
    self.deadlines.peek().map(|item| {
      let now_date_time = Utc::now();
      let now_instant = Instant::now();
      let duration = item.deadline
        .signed_duration_since(now_date_time)
        .to_std().ok().unwrap_or(Duration::ZERO);
      now_instant + duration
    })
  }

  pub async fn next(&mut self) -> Option<N> {
    loop {
      let next_deadline = match self.next_deadline() {
        Some(instant) => Either::Left(sleep_until(instant)),
        None => Either::Right(pending())
      };

      let result = tokio::select! {
        item = self.reciever.recv() => Err(self.deadlines.push(item?)),
        () = next_deadline => Ok(self.deadlines.pop().expect("infallible"))
      };

      match result {
        Err(()) => continue,
        Ok(TimerItem { value, .. }) => break Some(value)
      };
    }
  }

  pub fn push(&mut self, value: N, deadline: DateTime<Utc>) {
    self.deadlines.push(TimerItem { deadline, value });
  }
}

#[derive(Debug)]
struct TimerItem<N> {
  deadline: DateTime<Utc>,
  value: N
}

impl<N> Eq for TimerItem<N> {}

impl<N> PartialEq for TimerItem<N> {
  fn eq(&self, other: &Self) -> bool {
    self.deadline == other.deadline
  }
}

impl<N> Ord for TimerItem<N> {
  fn cmp(&self, other: &Self) -> Ordering {
    self.deadline.cmp(&other.deadline).reverse()
  }
}

impl<N> PartialOrd for TimerItem<N> {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}
