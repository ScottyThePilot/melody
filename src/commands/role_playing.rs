use crate::prelude::*;
use crate::feature::dice_roll::Roll;
use crate::utils::Blockify;
use super::{MelodyContext, CommandState};



#[poise::command(
  slash_command,
  description_localized("en-US", "Rolls a configurable dice"),
  custom_data = CommandState::new()
    .usage_localized("en-US", ["/roll <dice notation>"])
    .examples_localized("en-US", [
      "/roll '3d20'",
      "/roll 'd20 + 2 advantage'",
      "/roll 'd20 - 2 disadvantage'",
      "/roll '6d6 max 2'",
      "/roll '6d6 min 2'",
      "/roll '2d8 + 3'",
      "/roll 'coin'"
    ])
)]
pub async fn roll(
  ctx: MelodyContext<'_>,
  #[description_localized("en-US", "The notation for the dice roll to be made")]
  #[max_length = 1000]
  notation: String
) -> MelodyResult {
  let response = match notation.parse::<Roll>() {
    Ok(roll) => {
      let roll_message = roll.execute().to_string();
      if roll_message.len() > 2000 {
        "The resulting message was too long to send...".to_owned()
      } else {
        roll_message
      }
    },
    Err(error) => {
      Blockify(format_args!("Error: {error}")).to_string()
    }
  };

  ctx.reply(response).await.context("failed to send reply")?;
  Ok(())
}
