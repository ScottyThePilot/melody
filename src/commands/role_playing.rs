use crate::MelodyResult;
use crate::blueprint::*;
use crate::data::Core;
use crate::feature::dice_roll::Roll;
use crate::utils::Blockify;



pub const ROLL: BlueprintCommand = blueprint_command! {
  name: "roll",
  description: "Rolls a configurable dice",
  usage: ["/roll <dice notation>"],
  examples: [
    "/roll '3d20'",
    "/roll 'd20 + 2 advantage'",
    "/roll 'd20 - 2 disadvantage'",
    "/roll '6d6 max 2'",
    "/roll '6d6 min 2'",
    "/roll '2d8 + 3'",
    "/roll 'coin'"
  ],
  allow_in_dms: true,
  arguments: [
    blueprint_argument!(String {
      name: "dice",
      description: "The notation for the dice roll to be made",
      required: true
    })
  ],
  function: roll
};

#[command_attr::hook]
async fn roll(core: Core, args: BlueprintCommandArgs) -> MelodyResult {
  match resolve_arguments::<String>(args.option_values)?.parse::<Roll>() {
    Ok(roll) => {
      let roll_message = roll.execute().to_string();
      let response = if roll_message.len() > 2000 {
        "The resulting message was too long to send...".to_owned()
      } else {
        roll_message
      };

      BlueprintCommandResponse::new(response)
        .send(&core, &args.interaction).await
    },
    Err(error) => {
      let response = Blockify(format_args!("Error: {error}")).to_string();
      BlueprintCommandResponse::new_ephemeral(response)
        .send(&core, &args.interaction).await
    }
  }
}
