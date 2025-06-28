use serenity::all::ChannelId;

pub trait ChannelIdExt {
    fn channel_id_link(self) -> String;
}

impl ChannelIdExt for ChannelId {
    fn channel_id_link(self) -> String {
        format!("<#{}>", self)
    }
}
