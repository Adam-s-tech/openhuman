//! Portable provider exports.
//!
//! Provider transports, and the provider-independent remote control and
//! in-chat approvals that used to be Telegram glue, belong to `tinychannels`.

pub use tinychannels::providers::email_channel;
pub use tinychannels::providers::lark;
pub use tinychannels::providers::{
    dingtalk, discord, imessage, irc, linq, mattermost, qq, signal, slack, telegram, whatsapp,
    yuanbao,
};
#[cfg(feature = "whatsapp-web")]
pub use tinychannels::providers::whatsapp_web;
