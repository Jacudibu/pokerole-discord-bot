use crate::shared::errors::DataParsingError;
use crate::shared::game_data::parser::custom_data::custom_item::CustomItem;
use crate::shared::game_data::pokerole_data::raw_item::RawPokeroleItem;
use std::ops::Not;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Item {
    pub name: String,
    pub price: Option<u16>,
    pub description: String,
    pub category: String,
    pub single_use: bool,
    pub health_restored: Option<u8>,
}

impl Item {
    pub fn build_string(&self) -> impl Into<String> {
        let mut result: String = std::format!("### {}\n", &self.name);

        if let Some(health_restored) = &self.health_restored {
            result.push_str(&format!("**Health Restored**: {}\n", health_restored));
        }
        if let Some(price) = &self.price {
            result.push_str(&format!("**Price**: {}\n", price));
        }
        if self.category.is_empty().not() {
            result.push_str(&format!("**Category**: {}\n", self.category));
        }
        result.push_str(&self.description);

        result
    }
}

impl Item {
    pub fn from_pokerole(raw: RawPokeroleItem) -> Self {
        Item {
            name: raw.name,
            price: Item::parse_price(raw.pmd_price, raw.trainer_price).unwrap_or_default(),
            description: raw.description,
            category: Item::parse_category(raw.pocket, raw.category),
            single_use: raw.one_use,
            health_restored: raw.health_restored,
        }
    }

    pub fn from_custom_data(raw: CustomItem) -> Result<Self, DataParsingError> {
        Ok(Item {
            name: raw.name,
            price: Item::parse_price(raw.price, None)?,
            description: raw.description,
            category: raw.category,
            single_use: raw.single_use,
            health_restored: raw.health_restored,
        })
    }

    fn parse_price(pmd: Option<u16>, trainer: Option<String>) -> Result<Option<u16>, String> {
        if let Some(pmd_price) = pmd {
            if pmd_price == 0 {
                return Ok(None);
            }

            return Ok(pmd);
        }

        Item::parse_trainer_price(trainer)
    }

    fn parse_trainer_price(raw: Option<String>) -> Result<Option<u16>, String> {
        if let Some(some_raw) = raw {
            return match u16::from_str(&some_raw) {
                Ok(parsed) => Ok(Some(parsed)),
                Err(e) => Err(format!("Unable to parse trainer price: {e}")),
            };
        }

        Ok(None)
    }

    fn parse_category(raw_pocket: String, raw_category: String) -> String {
        if raw_category.is_empty() {
            return raw_category;
        }

        raw_pocket
    }
}
