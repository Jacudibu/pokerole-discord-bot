use crate::Error;
use crate::shared::{PoiseContext, SerenityContext};
use serenity::all::{ChannelId, CreateAllowedMentions, CreateMessage, GetMessages, GuildId, User};
use sqlx::{Pool, Sqlite};
use std::fmt;
use std::fmt::Formatter;

#[derive(PartialEq)]
pub enum ActionType {
    Initialization,
    Reward,
    Payment,
    BackpackUpgrade,
    HiddenAbilityUnlock,
    TradeOutgoing,
    TradeIncoming,
    WalletChange,
    WalletPayment,
    WalletWithdrawal,
    Undo,
    Spar,
    NewPlayerCombatTutorial,
    NewPlayerTour,
    WalletEdit,
    CharacterEdit,
    CharacterStatReset,
    CharacterRetirement,
    CharacterUnRetirement,
    TerastallizationUnlock,
    StoreGMExperience,
    UseGMExperience,
    RuleUpdate,
    RuleDelete,
    RuleClone,
    DoNotLog,
}

impl fmt::Display for ActionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            ActionType::Initialization => "🌟 [Init]",
            ActionType::Reward => "✨ [Reward]",
            ActionType::BackpackUpgrade => "🎒 [Upgrade]",
            ActionType::HiddenAbilityUnlock => "💊 [HA Unlock]",
            ActionType::Payment => "💰 [Payment]",
            ActionType::TradeOutgoing => "➡️ [Trade]",
            ActionType::TradeIncoming => "⬅️ [Trade]",
            ActionType::WalletChange => "👛 [Wallet]",
            ActionType::WalletPayment => "👛⬅️ [Payment]",
            ActionType::WalletWithdrawal => "👛➡️ [Withdrawal]",
            ActionType::Undo => "↩️ [Undo]",
            ActionType::Spar => "🤺 [Spar]",
            ActionType::NewPlayerCombatTutorial => "⚔️ [Combat Tutorial]",
            ActionType::NewPlayerTour => "🎫 [Tour]",
            ActionType::WalletEdit => "📝 [Edit]",
            ActionType::CharacterEdit => "📝 [Edit]",
            ActionType::CharacterStatReset => "📝 [Edit]",
            ActionType::CharacterRetirement => "💤 [Retirement]",
            ActionType::CharacterUnRetirement => "⏰ [UnRetirement]",
            ActionType::TerastallizationUnlock => "💎 [Terastallization Unlock]",
            ActionType::StoreGMExperience => "🏦⬅️ [GM Experience]",
            ActionType::UseGMExperience => "🏦➡️ [GM Experience]",
            ActionType::RuleUpdate => "⚖️🌟 [Rule Update]",
            ActionType::RuleDelete => "⚖️❌ [Rule Deletion]",
            ActionType::RuleClone => "⚖️⚖️⚖️ [Rule Clone]",
            ActionType::DoNotLog => "",
        })
    }
}

/// Necessary arguments for [log_action].
///
/// Use either [LogActionArguments::triggered_by_bot] or [LogActionArguments::triggered_by_user] to instantiate these.
pub struct LogActionArguments<'a> {
    pub author: Option<&'a User>,
    pub channel_id: Option<ChannelId>,
    pub guild_id: Option<GuildId>,
    pub context: &'a SerenityContext,
    pub database: &'a Pool<Sqlite>,
}

impl<'a> LogActionArguments<'a> {
    pub fn triggered_by_bot(context: &'a SerenityContext, database: &'a Pool<Sqlite>) -> Self {
        LogActionArguments {
            author: None,
            channel_id: None,
            guild_id: None,
            context,
            database,
        }
    }

    pub fn triggered_by_user(context: &'a PoiseContext) -> Self {
        LogActionArguments {
            author: Some(context.author()),
            channel_id: Some(context.channel_id()),
            guild_id: context.guild_id(),
            context: context.serenity_context(),
            database: &context.data().database,
        }
    }
}

pub async fn log_action<'a>(
    action_type: &ActionType,
    args: LogActionArguments<'a>,
    message: impl Into<String>,
) -> Result<(), Error> {
    // let guild_id = guild_id.expect("should only be called in guild_only").get() as i64;
    let Some(guild_id) = args.guild_id else {
        // TODO: Is that ever the case?
        return Ok(());
    };
    let guild_id_value = guild_id.get() as i64;

    let record = sqlx::query!(
        "SELECT action_log_channel_id FROM guild WHERE id = ?",
        guild_id_value
    )
    .fetch_one(args.database)
    .await;

    if let Some(channel_id) = args.channel_id {
        if let Some(author) = args.author {
            // If either of those is none, it was an automated action by the bot (e.g. retirement after user left server)

            let origin = match channel_id
                .messages(args.context, GetMessages::new().limit(1))
                .await
            {
                Ok(messages) => match messages.first() {
                    None => String::new(),
                    Some(m) => format!(" in {}", m.id.link(m.channel_id, Some(guild_id))),
                },
                Err(_) => String::new(),
            };

            if let Ok(record) = record {
                if let Some(action_log_channel_id) = record.action_log_channel_id {
                    let channel_id = ChannelId::from(action_log_channel_id as u64);
                    channel_id
                        .send_message(
                            args.context,
                            CreateMessage::new()
                                .content(std::format!(
                                    "{} {} (triggered by {}{})",
                                    action_type,
                                    message.into(),
                                    author,
                                    origin
                                ))
                                .allowed_mentions(CreateAllowedMentions::new().empty_users()),
                        )
                        .await?;
                }
            }
        }
    }

    Ok(())
}
