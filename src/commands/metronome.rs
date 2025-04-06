use crate::commands::Error;
use crate::shared;
use crate::shared::PoiseContext;

/// Use the most randomest of moves!
#[poise::command(slash_command)]
pub async fn metronome(ctx: PoiseContext<'_>) -> Result<(), Error> {
    let game_data = ctx.data().game.get_by_context(&ctx).await;

    ctx.say(shared::metronome::get_metronome_text(&game_data))
        .await?;
    Ok(())
}
