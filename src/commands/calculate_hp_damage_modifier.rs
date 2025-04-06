use crate::commands::Error;
use crate::shared::PoiseContext;

/// Use this to calculate your die count for reversal and other HP based moves
#[poise::command(slash_command)]
pub async fn calculate_hp_damage_modifier(
    ctx: PoiseContext<'_>,
    #[description = "What's your max HP?"] max_hp: u8,
) -> Result<(), Error> {
    ctx.say(build_string(max_hp)).await?;
    Ok(())
}

fn build_string(max_hp: u8) -> String {
    let segments = get_hp_segments(max_hp);

    std::format!(
        "With {} max hp, you have the following die thresholds depending on your current hp: ```
{:>2} - {:>2} -> 1 dice
{:>2} - {:>2} -> 2 dice
{:>2} - {:>2} -> 3 dice
{:>2} - {:>2} -> 4 dice
{:>2} - {:>2} -> 5 dice
```",
        max_hp,
        segments[0],
        max_hp - 1,
        segments[1],
        segments[0],
        segments[2],
        segments[1],
        segments[3],
        segments[2],
        1,
        segments[3],
    )
}

fn get_hp_segments(max_hp: u8) -> [u8; 4] {
    let segment_size = max_hp as f32 * 0.2;
    [
        (segment_size * 4.0).ceil() as u8,
        (segment_size * 3.0).ceil() as u8,
        (segment_size * 2.0).ceil() as u8,
        (segment_size * 1.0).ceil() as u8,
    ]
}
