use std::time::Duration;

use crossbeam_channel::Sender;
use maildir::Maildir as ExtMaildir;
use serde_derive::Deserialize;

use crate::appearance::Appearance;
use crate::blocks::{Block, ConfigBlock, Update};
use crate::de::deserialize_duration;
use crate::errors::*;
use crate::input::I3BarEvent;
use crate::scheduler::Task;
use crate::widgets::text::TextWidget;
use crate::widgets::{I3BarWidget, State};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MailType {
    New,
    Cur,
    All,
}

impl MailType {
    fn count_mail(&self, maildir: &ExtMaildir) -> usize {
        match self {
            MailType::New => maildir.count_new(),
            MailType::Cur => maildir.count_cur(),
            MailType::All => maildir.count_new() + maildir.count_cur(),
        }
    }
}

impl Default for MailType {
    fn default() -> MailType {
        MailType::New
    }
}

pub struct Maildir {
    id: usize,
    text: TextWidget,
    update_interval: Duration,
    inboxes: Vec<String>,
    threshold_warning: usize,
    threshold_critical: usize,
    display_type: MailType,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct MaildirConfig {
    /// Update interval in seconds
    #[serde(
        default = "MaildirConfig::default_interval",
        deserialize_with = "deserialize_duration"
    )]
    pub interval: Duration,
    pub inboxes: Vec<String>,
    #[serde(default = "MaildirConfig::default_threshold_warning")]
    pub threshold_warning: usize,
    #[serde(default = "MaildirConfig::default_threshold_critical")]
    pub threshold_critical: usize,
    #[serde(default)]
    pub display_type: MailType,
    #[serde(default = "MaildirConfig::default_icon")]
    pub icon: bool,
}

impl MaildirConfig {
    fn default_interval() -> Duration {
        Duration::from_secs(5)
    }
    fn default_threshold_warning() -> usize {
        1
    }
    fn default_threshold_critical() -> usize {
        10
    }
    fn default_icon() -> bool {
        true
    }
}

impl ConfigBlock for Maildir {
    type Config = MaildirConfig;

    fn new(
        id: usize,
        block_config: Self::Config,
        appearance: Appearance,
        _tx_update_request: Sender<Task>,
    ) -> Result<Self> {
        let widget = TextWidget::new(id, appearance).with_text("");
        Ok(Maildir {
            id,
            update_interval: block_config.interval,
            text: if block_config.icon {
                widget.with_icon("mail")
            } else {
                widget
            },
            inboxes: block_config.inboxes,
            threshold_warning: block_config.threshold_warning,
            threshold_critical: block_config.threshold_critical,
            display_type: block_config.display_type,
        })
    }
}

impl Block for Maildir {
    fn update(&mut self) -> Result<Option<Update>> {
        let mut newmails = 0;
        for inbox in &self.inboxes {
            let isl: &str = &inbox[..];
            let maildir = ExtMaildir::from(isl);
            newmails += self.display_type.count_mail(&maildir)
        }
        let mut state = State::Idle;
        if newmails >= self.threshold_critical {
            state = State::Critical;
        } else if newmails >= self.threshold_warning {
            state = State::Warning;
        }
        self.text.set_state(state);
        self.text.set_text(format!("{}", newmails));
        Ok(Some(self.update_interval.into()))
    }

    fn view(&self) -> Vec<&dyn I3BarWidget> {
        vec![&self.text]
    }

    fn click(&mut self, _: &I3BarEvent) -> Result<()> {
        Ok(())
    }

    fn id(&self) -> usize {
        self.id
    }
}
