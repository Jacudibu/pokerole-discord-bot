use crate::commands::autocompletion::autocomplete_move;
use crate::commands::Error;
use crate::shared::game_data::r#move::Move;
use crate::shared::helpers;
use crate::shared::PoiseContext;
use poise::CreateReply;
use serenity::all::CreateActionRow;

/// Display a move
#[poise::command(slash_command, rename = "move")]
pub async fn poke_move(
    ctx: PoiseContext<'_>,
    #[description = "Which move?"]
    #[rename = "move"]
    #[autocomplete = "autocomplete_move"]
    name: String,
) -> Result<(), Error> {
    let game_data = ctx.data().game.get_by_context(&ctx).await;

    if let Some(poke_move) = game_data.moves.get(&name.to_lowercase()) {
        if poke_move.name == "Metronome" {
            execute_metronome(ctx, poke_move).await?;
        } else {
            ctx.say(poke_move.build_string()).await?;
        }
    } else {
        ctx.send(CreateReply::default()
            .content(std::format!("Unable to find a move named **{}**, sorry! If that wasn't a typo, maybe it isn't implemented yet?", name))
            .ephemeral(true)
        ).await?;
    }

    Ok(())
}

async fn execute_metronome<'a>(ctx: PoiseContext<'a>, poke_move: &Move) -> Result<(), Error> {
    let reply = ctx
        .send(
            CreateReply::default()
                .content(poke_move.build_string())
                .components(vec![CreateActionRow::Buttons(vec![
                    helpers::create_button("Use Metronome", "metronome", false),
                ])]),
        )
        .await?;

    reply.message().await?;
    Ok(())
}
