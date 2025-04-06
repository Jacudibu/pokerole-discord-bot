use crate::commands::autocompletion::autocomplete_potion;
use crate::commands::Error;
use crate::shared::PoiseContext;
use poise::CreateReply;

/// List potion effects and crafting recipes
#[poise::command(slash_command)]
pub async fn potion(
    ctx: PoiseContext<'_>,
    #[description = "Which item?"]
    #[rename = "name"]
    #[autocomplete = "autocomplete_potion"]
    name: String,
) -> Result<(), Error> {
    let game_data = ctx.data().game.get_by_context(&ctx).await;
    if let Some(potion) = game_data.potions.get(&name.to_lowercase()) {
        ctx.say(potion.build_string()).await?;
    } else {
        ctx.send(
            CreateReply::default()
                .content(std::format!(
                    "Unable to find a potion named **{}**, sorry!",
                    name
                ))
                .ephemeral(true),
        )
        .await?;
    }

    Ok(())
}
