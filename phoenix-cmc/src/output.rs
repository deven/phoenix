use crate::name::Name;
use crate::sendlist::Sendlist;
use crate::telnet::Telnet;
use crate::timestamp::Timestamp;
use crate::types::{ArcStr, OutputClass, OutputType};
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait OutputObj: Send + Sync {
    fn output_type(&self) -> OutputType;
    fn output_class(&self) -> OutputClass;
    fn time(&self) -> Timestamp;
    async fn output(&self, telnet: &Arc<Telnet>);
}

#[derive(Debug, Clone)]
pub struct Text {
    pub text: String,
    pub time: Timestamp,
}

impl Text {
    pub fn new(text: String) -> Self {
        Self {
            text,
            time: Timestamp::new(),
        }
    }
}

#[async_trait]
impl OutputObj for Text {
    fn output_type(&self) -> OutputType {
        OutputType::TextOutput
    }

    fn output_class(&self) -> OutputClass {
        OutputClass::TextClass
    }

    fn time(&self) -> Timestamp {
        self.time
    }

    async fn output(&self, telnet: &Arc<Telnet>) {
        telnet.output(&self.text).await;
    }
}

#[derive(Debug, Clone)]
pub struct Message {
    pub output_type: OutputType,
    pub from: Arc<Name>,
    pub to: Arc<Sendlist>,
    pub text: ArcStr,
    pub time: Timestamp,
}

impl Message {
    pub fn new(
        output_type: OutputType,
        sender: Arc<Name>,
        dest: Arc<Sendlist>,
        msg: impl Into<ArcStr>,
    ) -> Self {
        Self {
            output_type,
            from: sender,
            to: dest,
            text: msg.into(),
            time: Timestamp::new(),
        }
    }
}

#[async_trait]
impl OutputObj for Message {
    fn output_type(&self) -> OutputType {
        self.output_type
    }

    fn output_class(&self) -> OutputClass {
        OutputClass::MessageClass
    }

    fn time(&self) -> Timestamp {
        self.time
    }

    async fn output(&self, telnet: &Arc<Telnet>) {
        telnet
            .print_message(
                self.output_type,
                self.time,
                &self.from,
                &self.to,
                &self.text,
            )
            .await;
    }
}

#[derive(Debug, Clone)]
pub struct EntryNotify {
    pub name: Arc<Name>,
    pub time: Timestamp,
}

impl EntryNotify {
    pub fn new(who: Arc<Name>) -> Self {
        Self {
            name: who,
            time: Timestamp::new(),
        }
    }
}

#[async_trait]
impl OutputObj for EntryNotify {
    fn output_type(&self) -> OutputType {
        OutputType::EntryOutput
    }

    fn output_class(&self) -> OutputClass {
        OutputClass::NotificationClass
    }

    fn time(&self) -> Timestamp {
        self.time
    }

    async fn output(&self, telnet: &Arc<Telnet>) {
        let name = &self.name.name;
        let blurb = &self.name.blurb;
        let stamp = &self.time.stamp();
        telnet
            .output(&format!(
                "*** {name}{blurb} has entered Phoenix! [{stamp}] ***\n"
            ))
            .await;
    }
}

#[derive(Debug, Clone)]
pub struct ExitNotify {
    pub name: Arc<Name>,
    pub time: Timestamp,
}

impl ExitNotify {
    pub fn new(who: Arc<Name>) -> Self {
        Self {
            name: who,
            time: Timestamp::new(),
        }
    }
}

#[async_trait]
impl OutputObj for ExitNotify {
    fn output_type(&self) -> OutputType {
        OutputType::ExitOutput
    }

    fn output_class(&self) -> OutputClass {
        OutputClass::NotificationClass
    }

    fn time(&self) -> Timestamp {
        self.time
    }

    async fn output(&self, telnet: &Arc<Telnet>) {
        let name = &self.name.name;
        let blurb = &self.name.blurb;
        let stamp = &self.time.stamp();
        telnet
            .output(&format!(
                "*** {name}{blurb} has left Phoenix! [{stamp}] ***\n"
            ))
            .await;
    }
}

#[derive(Debug, Clone)]
pub struct TransferNotify {
    pub name: Arc<Name>,
    pub time: Timestamp,
}

impl TransferNotify {
    pub fn new(who: Arc<Name>) -> Self {
        Self {
            name: who,
            time: Timestamp::new(),
        }
    }
}

#[async_trait]
impl OutputObj for TransferNotify {
    fn output_type(&self) -> OutputType {
        OutputType::TransferOutput
    }

    fn output_class(&self) -> OutputClass {
        OutputClass::NotificationClass
    }

    fn time(&self) -> Timestamp {
        self.time
    }

    async fn output(&self, telnet: &Arc<Telnet>) {
        let name = &self.name.name;
        let blurb = &self.name.blurb;
        let stamp = &self.time.stamp();
        telnet
            .output(&format!(
                "*** {name}{blurb} has transferred to new connection. [{stamp}] ***\n"
            ))
            .await;
    }
}

#[derive(Debug, Clone)]
pub struct AttachNotify {
    pub name: Arc<Name>,
    pub time: Timestamp,
}

impl AttachNotify {
    pub fn new(who: Arc<Name>) -> Self {
        Self {
            name: who,
            time: Timestamp::new(),
        }
    }
}

#[async_trait]
impl OutputObj for AttachNotify {
    fn output_type(&self) -> OutputType {
        OutputType::AttachOutput
    }

    fn output_class(&self) -> OutputClass {
        OutputClass::NotificationClass
    }

    fn time(&self) -> Timestamp {
        self.time
    }

    async fn output(&self, telnet: &Arc<Telnet>) {
        let name = &self.name.name;
        let blurb = &self.name.blurb;
        let stamp = &self.time.stamp();
        telnet
            .output(&format!(
                "*** {name}{blurb} is now attached. [{stamp}] ***\n"
            ))
            .await;
    }
}

#[derive(Debug, Clone)]
pub struct DetachNotify {
    pub name: Arc<Name>,
    pub intentional: bool,
    pub time: Timestamp,
}

impl DetachNotify {
    pub fn new(who: Arc<Name>, i: bool) -> Self {
        Self {
            name: who,
            intentional: i,
            time: Timestamp::new(),
        }
    }
}

#[async_trait]
impl OutputObj for DetachNotify {
    fn output_type(&self) -> OutputType {
        OutputType::DetachOutput
    }

    fn output_class(&self) -> OutputClass {
        OutputClass::NotificationClass
    }

    fn time(&self) -> Timestamp {
        self.time
    }

    async fn output(&self, telnet: &Arc<Telnet>) {
        let name = &self.name.name;
        let blurb = &self.name.blurb;
        let stamp = &self.time.stamp();
        let intentionally = if self.intentional {
            "intentionally"
        } else {
            "accidentally"
        };
        telnet
            .output(&format!(
                "*** {name}{blurb} has {intentionally} detached. [{stamp}] ***\n"
            ))
            .await;
    }
}

#[derive(Debug, Clone)]
pub struct HereNotify {
    pub name: Arc<Name>,
    pub time: Timestamp,
}

impl HereNotify {
    pub fn new(who: Arc<Name>) -> Self {
        Self {
            name: who,
            time: Timestamp::new(),
        }
    }
}

#[async_trait]
impl OutputObj for HereNotify {
    fn output_type(&self) -> OutputType {
        OutputType::HereOutput
    }

    fn output_class(&self) -> OutputClass {
        OutputClass::NotificationClass
    }

    fn time(&self) -> Timestamp {
        self.time
    }

    async fn output(&self, telnet: &Arc<Telnet>) {
        let name = &self.name.name;
        let blurb = &self.name.blurb;
        let stamp = &self.time.stamp();
        telnet
            .output(&format!("*** {name}{blurb} is now here. [{stamp}] ***\n"))
            .await;
    }
}

#[derive(Debug, Clone)]
pub struct AwayNotify {
    pub name: Arc<Name>,
    pub time: Timestamp,
}

impl AwayNotify {
    pub fn new(who: Arc<Name>) -> Self {
        Self {
            name: who,
            time: Timestamp::new(),
        }
    }
}

#[async_trait]
impl OutputObj for AwayNotify {
    fn output_type(&self) -> OutputType {
        OutputType::AwayOutput
    }

    fn output_class(&self) -> OutputClass {
        OutputClass::NotificationClass
    }

    fn time(&self) -> Timestamp {
        self.time
    }

    async fn output(&self, telnet: &Arc<Telnet>) {
        let name = &self.name.name;
        let blurb = &self.name.blurb;
        let stamp = &self.time.stamp();
        telnet
            .output(&format!("*** {name}{blurb} is now away. [{stamp}] ***\n"))
            .await;
    }
}

#[derive(Debug, Clone)]
pub struct BusyNotify {
    pub name: Arc<Name>,
    pub time: Timestamp,
}

impl BusyNotify {
    pub fn new(who: Arc<Name>) -> Self {
        Self {
            name: who,
            time: Timestamp::new(),
        }
    }
}

#[async_trait]
impl OutputObj for BusyNotify {
    fn output_type(&self) -> OutputType {
        OutputType::BusyOutput
    }

    fn output_class(&self) -> OutputClass {
        OutputClass::NotificationClass
    }

    fn time(&self) -> Timestamp {
        self.time
    }

    async fn output(&self, telnet: &Arc<Telnet>) {
        let name = &self.name.name;
        let blurb = &self.name.blurb;
        let stamp = &self.time.stamp();
        telnet
            .output(&format!("*** {name}{blurb} is now busy. [{stamp}] ***\n"))
            .await;
    }
}

#[derive(Debug, Clone)]
pub struct GoneNotify {
    pub name: Arc<Name>,
    pub time: Timestamp,
}

impl GoneNotify {
    pub fn new(who: Arc<Name>) -> Self {
        Self {
            name: who,
            time: Timestamp::new(),
        }
    }
}

#[async_trait]
impl OutputObj for GoneNotify {
    fn output_type(&self) -> OutputType {
        OutputType::GoneOutput
    }

    fn output_class(&self) -> OutputClass {
        OutputClass::NotificationClass
    }

    fn time(&self) -> Timestamp {
        self.time
    }

    async fn output(&self, telnet: &Arc<Telnet>) {
        let name = &self.name.name;
        let blurb = &self.name.blurb;
        let stamp = &self.time.stamp();
        telnet
            .output(&format!("*** {name}{blurb} is now gone. [{stamp}] ***\n"))
            .await;
    }
}

#[derive(Debug, Clone)]
pub struct CreateNotify {
    pub discussion_name: ArcStr,
    pub discussion_title: ArcStr,
    pub is_public: bool,
    pub creator: Arc<Name>,
    pub time: Timestamp,
}

impl CreateNotify {
    pub fn new(disc_name: ArcStr, disc_title: ArcStr, is_public: bool, creator: Arc<Name>) -> Self {
        Self {
            discussion_name: disc_name,
            discussion_title: disc_title,
            is_public,
            creator,
            time: Timestamp::new(),
        }
    }
}

#[async_trait]
impl OutputObj for CreateNotify {
    fn output_type(&self) -> OutputType {
        OutputType::CreateOutput
    }

    fn output_class(&self) -> OutputClass {
        OutputClass::NotificationClass
    }

    fn time(&self) -> Timestamp {
        self.time
    }

    async fn output(&self, telnet: &Arc<Telnet>) {
        let name = &self.creator.name;
        let blurb = &self.creator.blurb;
        let disc = &self.discussion_name;
        let title = &self.discussion_title;
        let stamp = &self.time.stamp();
        if self.is_public {
            telnet
                .output(&format!(
                    "*** {name}{blurb} has created discussion {disc}, \"{title}\". [{stamp}] ***\n"
                ))
                .await;
        } else {
            telnet
                .output(&format!(
                    "*** {name}{blurb} has created private discussion {disc}. [{stamp}] ***\n"
                ))
                .await;
        }
    }
}

#[derive(Debug, Clone)]
pub struct DestroyNotify {
    pub discussion_name: ArcStr,
    pub name: Arc<Name>,
    pub time: Timestamp,
}

impl DestroyNotify {
    pub fn new(disc_name: ArcStr, who: Arc<Name>) -> Self {
        Self {
            discussion_name: disc_name,
            name: who,
            time: Timestamp::new(),
        }
    }
}

#[async_trait]
impl OutputObj for DestroyNotify {
    fn output_type(&self) -> OutputType {
        OutputType::DestroyOutput
    }

    fn output_class(&self) -> OutputClass {
        OutputClass::NotificationClass
    }

    fn time(&self) -> Timestamp {
        self.time
    }

    async fn output(&self, telnet: &Arc<Telnet>) {
        let name = &self.name.name;
        let blurb = &self.name.blurb;
        let disc = &self.discussion_name;
        let stamp = &self.time.stamp();
        telnet
            .output(&format!(
                "*** {name}{blurb} has destroyed discussion {disc}. [{stamp}] ***\n"
            ))
            .await;
    }
}

#[derive(Debug, Clone)]
pub struct JoinNotify {
    pub discussion_name: ArcStr,
    pub name: Arc<Name>,
    pub time: Timestamp,
}

impl JoinNotify {
    pub fn new(disc_name: ArcStr, who: Arc<Name>) -> Self {
        Self {
            discussion_name: disc_name,
            name: who,
            time: Timestamp::new(),
        }
    }
}

#[async_trait]
impl OutputObj for JoinNotify {
    fn output_type(&self) -> OutputType {
        OutputType::JoinOutput
    }

    fn output_class(&self) -> OutputClass {
        OutputClass::NotificationClass
    }

    fn time(&self) -> Timestamp {
        self.time
    }

    async fn output(&self, telnet: &Arc<Telnet>) {
        let name = &self.name.name;
        let blurb = &self.name.blurb;
        let disc = &self.discussion_name;
        let stamp = &self.time.stamp();
        telnet
            .output(&format!(
                "*** {name}{blurb} has joined discussion {disc}. [{stamp}] ***\n"
            ))
            .await;
    }
}

#[derive(Debug, Clone)]
pub struct QuitNotify {
    pub discussion_name: ArcStr,
    pub name: Arc<Name>,
    pub time: Timestamp,
}

impl QuitNotify {
    pub fn new(disc_name: ArcStr, who: Arc<Name>) -> Self {
        Self {
            discussion_name: disc_name,
            name: who,
            time: Timestamp::new(),
        }
    }
}

#[async_trait]
impl OutputObj for QuitNotify {
    fn output_type(&self) -> OutputType {
        OutputType::QuitOutput
    }

    fn output_class(&self) -> OutputClass {
        OutputClass::NotificationClass
    }

    fn time(&self) -> Timestamp {
        self.time
    }

    async fn output(&self, telnet: &Arc<Telnet>) {
        let name = &self.name.name;
        let blurb = &self.name.blurb;
        let disc = &self.discussion_name;
        let stamp = &self.time.stamp();
        telnet
            .output(&format!(
                "*** {name}{blurb} has quit discussion {disc}. [{stamp}] ***\n"
            ))
            .await;
    }
}

#[derive(Debug, Clone)]
pub struct PublicNotify {
    pub discussion_name: ArcStr,
    pub name: Arc<Name>,
    pub time: Timestamp,
}

impl PublicNotify {
    pub fn new(disc_name: ArcStr, who: Arc<Name>) -> Self {
        Self {
            discussion_name: disc_name,
            name: who,
            time: Timestamp::new(),
        }
    }
}

#[async_trait]
impl OutputObj for PublicNotify {
    fn output_type(&self) -> OutputType {
        OutputType::PublicOutput
    }

    fn output_class(&self) -> OutputClass {
        OutputClass::NotificationClass
    }

    fn time(&self) -> Timestamp {
        self.time
    }

    async fn output(&self, telnet: &Arc<Telnet>) {
        let name = &self.name.name;
        let blurb = &self.name.blurb;
        let disc = &self.discussion_name;
        let stamp = &self.time.stamp();
        telnet
            .output(&format!(
                "*** {name}{blurb} has made discussion {disc} public. [{stamp}] ***\n"
            ))
            .await;
    }
}

#[derive(Debug, Clone)]
pub struct PrivateNotify {
    pub discussion_name: ArcStr,
    pub name: Arc<Name>,
    pub time: Timestamp,
}

impl PrivateNotify {
    pub fn new(disc_name: ArcStr, who: Arc<Name>) -> Self {
        Self {
            discussion_name: disc_name,
            name: who,
            time: Timestamp::new(),
        }
    }
}

#[async_trait]
impl OutputObj for PrivateNotify {
    fn output_type(&self) -> OutputType {
        OutputType::PrivateOutput
    }

    fn output_class(&self) -> OutputClass {
        OutputClass::NotificationClass
    }

    fn time(&self) -> Timestamp {
        self.time
    }

    async fn output(&self, telnet: &Arc<Telnet>) {
        let name = &self.name.name;
        let blurb = &self.name.blurb;
        let disc = &self.discussion_name;
        let stamp = &self.time.stamp();
        telnet
            .output(&format!(
                "*** {name}{blurb} has made discussion {disc} private. [{stamp}] ***\n"
            ))
            .await;
    }
}

#[derive(Debug, Clone)]
pub struct PermitNotify {
    pub discussion_name: ArcStr,
    pub is_public: bool,
    pub name: Arc<Name>,
    pub is_explicit: bool,
    pub time: Timestamp,
}

impl PermitNotify {
    pub fn new(disc_name: ArcStr, public: bool, who: Arc<Name>, flag: bool) -> Self {
        Self {
            discussion_name: disc_name,
            is_public: public,
            name: who,
            is_explicit: flag,
            time: Timestamp::new(),
        }
    }
}

#[async_trait]
impl OutputObj for PermitNotify {
    fn output_type(&self) -> OutputType {
        OutputType::PermitOutput
    }

    fn output_class(&self) -> OutputClass {
        OutputClass::NotificationClass
    }

    fn time(&self) -> Timestamp {
        self.time
    }

    async fn output(&self, telnet: &Arc<Telnet>) {
        let name = &self.name.name;
        let blurb = &self.name.blurb;
        let disc = &self.discussion_name;
        let stamp = &self.time.stamp();
        if self.is_public {
            if self.is_explicit {
                telnet.output(&format!("*** {name}{blurb} has repermitted you to discussion {disc}. [{stamp}] ***\n")).await;
            } else {
                telnet.output(&format!("*** {name}{blurb} has explicitly permitted you to public discussion {disc}. [{stamp}] ***\n")).await;
            }
        } else {
            if self.is_explicit {
                telnet.output(&format!("*** {name}{blurb} has repermitted you to private discussion {disc}. [{stamp}] ***\n")).await;
            } else {
                telnet.output(&format!("*** {name}{blurb} has permitted you to private discussion {disc}. [{stamp}] ***\n")).await;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct DepermitNotify {
    pub discussion_name: ArcStr,
    pub is_public: bool,
    pub name: Arc<Name>,
    pub is_explicit: bool,
    pub removed: Option<Arc<Name>>,
    pub time: Timestamp,
}

impl DepermitNotify {
    pub fn new(
        disc_name: ArcStr,
        public: bool,
        who: Arc<Name>,
        flag: bool,
        removed_who: Option<Arc<Name>>,
    ) -> Self {
        Self {
            discussion_name: disc_name,
            is_public: public,
            name: who,
            is_explicit: flag,
            removed: removed_who,
            time: Timestamp::new(),
        }
    }
}

#[async_trait]
impl OutputObj for DepermitNotify {
    fn output_type(&self) -> OutputType {
        OutputType::DepermitOutput
    }

    fn output_class(&self) -> OutputClass {
        OutputClass::NotificationClass
    }

    fn time(&self) -> Timestamp {
        self.time
    }

    async fn output(&self, telnet: &Arc<Telnet>) {
        let session_name = telnet.session_name().await;
        let name = &self.name.name;
        let blurb = &self.name.blurb;
        let disc = &self.discussion_name;
        let stamp = &self.time.stamp();

        if self.is_public {
            if let Some(removed) = &self.removed {
                let removed_name = &removed.name;
                let removed_blurb = &removed.blurb;
                if removed_name.eq_ignore_ascii_case(&session_name) {
                    telnet.output(&format!("*** {name}{blurb} has depermitted and removed you from discussion {disc}. [{stamp}] ***\n")).await;
                } else {
                    telnet.output(&format!("*** {removed_name}{removed_blurb} has been removed from discussion {disc}. [{stamp}] ***\n")).await;
                }
            } else {
                telnet.output(&format!("*** {name}{blurb} has depermitted you from discussion {disc}. [{stamp}] ***\n")).await;
            }
        } else {
            if self.is_explicit {
                telnet.output(&format!("*** {name}{blurb} has explicitly depermitted you from private discussion {disc}. [{stamp}] ***\n")).await;
            } else {
                if let Some(removed) = &self.removed {
                    let removed_name = &self.removed.name;
                    let removed_blurb = &self.removed.blurb;
                    if removed_name.eq_ignore_ascii_case(&session_name) {
                        telnet.output(&format!("*** {name}{blurb} has depermitted and removed you from private discussion {disc}. [{stamp}] ***\n")).await;
                    } else {
                        telnet.output(&format!("*** {removed_name}{removed_blurb} has been removed from discussion {disc}. [{stamp}] ***\n")).await;
                    }
                } else {
                    telnet.output(&format!("*** {name}{blurb} has depermitted you from private discussion {disc}. [{stamp}] ***\n")).await;
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppointNotify {
    pub discussion_name: ArcStr,
    pub appointer: Arc<Name>,
    pub appointee: Arc<Name>,
    pub time: Timestamp,
}

impl AppointNotify {
    pub fn new(disc_name: ArcStr, who1: Arc<Name>, who2: Arc<Name>) -> Self {
        Self {
            discussion_name: disc_name,
            appointer: who1,
            appointee: who2,
            time: Timestamp::new(),
        }
    }
}

#[async_trait]
impl OutputObj for AppointNotify {
    fn output_type(&self) -> OutputType {
        OutputType::AppointOutput
    }

    fn output_class(&self) -> OutputClass {
        OutputClass::NotificationClass
    }

    fn time(&self) -> Timestamp {
        self.time
    }

    async fn output(&self, telnet: &Arc<Telnet>) {
        let session_name = telnet.session_name().await;
        let appointer_name = &self.appointer.name;
        let appointer_blurb = &self.appointer.blurb;
        let appointee_name = &self.appointee.name;
        let appointee_blurb = &self.appointee.blurb;
        let disc = &self.discussion_name;
        let stamp = &self.time.stamp();

        if self.appointee.name.eq_ignore_ascii_case(&session_name) {
            telnet.output(&format!("*** {appointer_name}{appointer_blurb} has appointed you as a moderator of discussion {disc}. [{stamp}] ***\n")).await;
        } else {
            telnet.output(&format!("*** {appointer_name}{appointer_blurb} has appointed {appointee_name}{appointee_blurb} as a moderator of discussion {disc}. [{stamp}] ***\n")).await;
        }
    }
}

#[derive(Debug, Clone)]
pub struct UnappointNotify {
    pub discussion_name: ArcStr,
    pub unappointer: Arc<Name>,
    pub unappointee: Arc<Name>,
    pub time: Timestamp,
}

impl UnappointNotify {
    pub fn new(disc_name: ArcStr, who1: Arc<Name>, who2: Arc<Name>) -> Self {
        Self {
            discussion_name: disc_name,
            unappointer: who1,
            unappointee: who2,
            time: Timestamp::new(),
        }
    }
}

#[async_trait]
impl OutputObj for UnappointNotify {
    fn output_type(&self) -> OutputType {
        OutputType::UnappointOutput
    }

    fn output_class(&self) -> OutputClass {
        OutputClass::NotificationClass
    }

    fn time(&self) -> Timestamp {
        self.time
    }

    async fn output(&self, telnet: &Arc<Telnet>) {
        let session_name = telnet.session_name().await;
        let unappointer_name = &self.unappointer.name;
        let unappointer_blurb = &self.unappointer.blurb;
        let unappointee_name = &self.unappointee.name;
        let unappointee_blurb = &self.unappointee.blurb;
        let disc = &self.discussion_name;
        let stamp = &self.time.stamp();

        if self.unappointee.name.eq_ignore_ascii_case(&session_name) {
            telnet.output(&format!("*** {unappointer_name}{unappointer_blurb} has unappointed you as a moderator of discussion {disc}. [{stamp}] ***\n")).await;
        } else {
            telnet.output(&format!("*** {unappointer_name}{unappointer_blurb} has unappointed {unappointee_name}{unappointee_blurb} as a moderator of discussion {disc}. [{stamp}] ***\n")).await;
        }
    }
}

#[derive(Debug, Clone)]
pub struct RenameNotify {
    pub oldname: ArcStr,
    pub newname: ArcStr,
    pub time: Timestamp,
}

impl RenameNotify {
    pub fn new(oldstr: impl Into<ArcStr>, newstr: impl Into<ArcStr>) -> Self {
        Self {
            oldname: oldstr.into(),
            newname: newstr.into(),
            time: Timestamp::new(),
        }
    }
}

#[async_trait]
impl OutputObj for RenameNotify {
    fn output_type(&self) -> OutputType {
        OutputType::RenameOutput
    }

    fn output_class(&self) -> OutputClass {
        OutputClass::NotificationClass
    }

    fn time(&self) -> Timestamp {
        self.time
    }

    async fn output(&self, telnet: &Arc<Telnet>) {
        let oldname = &self.oldname;
        let newname = &self.newname;
        let stamp = &self.time.stamp();
        telnet
            .output(&format!(
                "*** {oldname} has renamed to {newname}. [{stamp}] ***\n"
            ))
            .await;
    }
}

// Output stream for queuing output objects
#[derive(Debug, Clone)]
pub struct OutputStream {
    pub queue: tokio::sync::Mutex<Vec<Arc<dyn OutputObj>>>,
    pub acknowledged: std::sync::atomic::AtomicUsize,
    pub sent: std::sync::atomic::AtomicUsize,
}

impl OutputStream {
    pub fn new() -> Self {
        Self {
            queue: tokio::sync::Mutex::new(Vec::new()),
            acknowledged: std::sync::atomic::AtomicUsize::new(0),
            sent: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    pub async fn acknowledge(&self) {
        let sent = self.sent.load(std::sync::atomic::Ordering::Relaxed);
        let ack = self.acknowledged.load(std::sync::atomic::Ordering::Relaxed);
        if ack < sent {
            self.acknowledged
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
    }

    pub async fn attach(&self, telnet: &Arc<Telnet>) {
        self.sent.store(0, std::sync::atomic::Ordering::Relaxed);
        self.acknowledged
            .store(0, std::sync::atomic::Ordering::Relaxed);

        if telnet.acknowledge().await {
            while self.send_next(telnet).await {}
        }
    }

    pub async fn enqueue(&self, telnet: Option<&Arc<Telnet>>, out: Arc<dyn OutputObj>) {
        self.queue.lock().await.push(out);

        if let Some(telnet) = telnet {
            if telnet.acknowledge().await {
                while self.send_next(telnet).await {}
            } else if self.queue.lock().await.len() == 1 {
                self.send_next(telnet).await;
            }
        }
    }

    pub async fn unenqueue(&self, out: &dyn OutputObj) {
        let mut queue = self.queue.lock().await;
        queue.retain(|item| !std::ptr::eq(item.as_ref() as *const _, out as *const _));
    }

    pub async fn dequeue(&self) {
        let ack = self.acknowledged.load(std::sync::atomic::Ordering::Relaxed);
        if ack > 0 {
            let mut queue = self.queue.lock().await;
            let to_remove = ack.min(queue.len());
            queue.drain(..to_remove);
            self.acknowledged
                .fetch_sub(to_remove, std::sync::atomic::Ordering::Relaxed);
            self.sent
                .fetch_sub(to_remove, std::sync::atomic::Ordering::Relaxed);
        }
    }

    pub async fn send_next(&self, telnet: &Arc<Telnet>) -> bool {
        let queue = self.queue.lock().await;
        let sent = self.sent.load(std::sync::atomic::Ordering::Relaxed);

        if sent >= queue.len() {
            telnet.redraw_input().await;
            false
        } else {
            let out = queue[sent].clone();

            telnet.undraw_input().await;
            out.output(telnet).await;
            telnet.timing_mark().await;
            self.sent.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            true
        }
    }
}

impl Default for OutputStream {
    fn default() -> Self {
        Self::new()
    }
}
