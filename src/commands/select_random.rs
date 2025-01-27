use crate::commands::{Context, Error};
use rand::Rng;

/// Get randomized targets from a comma separated list of targets.
#[poise::command(slash_command)]
pub async fn select_random(
    ctx: Context<'_>,
    #[description = "How many targets?"] amount: u8,
    #[description = "name1, name2, name3, name4..."] targets: String,
) -> Result<(), Error> {
    let random_targets = get_randomized_elements(amount, targets);
    ctx.say(std::format!("**Targets**: {}", random_targets))
        .await?;
    Ok(())
}

fn get_randomized_elements(amount: u8, targets: String) -> String {
    let mut target_split: Vec<String> = targets
        .split(',')
        .map(|x| x.to_string())
        .collect::<Vec<String>>();
    let mut result = Vec::default();
    let mut rng = rand::rng();

    for _ in 0..amount {
        if target_split.is_empty() {
            break;
        }

        let index = rng.random_range(0..target_split.len());
        result.push(target_split.remove(index).trim().to_string());
    }

    result.join(", ")
}
