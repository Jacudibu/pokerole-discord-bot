use crate::commands::Error;
use crate::shared::{dice_rolls, PoiseContext};

/// Roll dice using a "1d6+4" style text query.
#[poise::command(slash_command)]
pub async fn r(
    ctx: PoiseContext<'_>,
    #[description = "1d6+5 will roll 1d6 and add 5."] query: String,
) -> Result<(), Error> {
    dice_rolls::execute_query(&ctx, &query).await
}

/// Roll dice by entering die amount, sides and flat addition manually.
#[poise::command(slash_command)]
pub async fn roll(
    ctx: PoiseContext<'_>,
    #[description = "How many dies?"]
    #[min = 1_u8]
    #[max = 100_u8]
    dice: Option<u8>,
    #[description = "How many sides?"]
    #[min = 2_u8]
    #[max = 100_u8]
    sides: Option<u8>,
    #[description = "Add a flat value to the result"]
    #[min = 0_u8]
    #[max = 100_u8]
    flat_addition: Option<u8>,
) -> Result<(), Error> {
    dice_rolls::roll(&ctx, dice, sides, flat_addition).await
}
