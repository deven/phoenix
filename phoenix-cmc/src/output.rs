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
    async fn output(&self, telnet: &mut Telnet);
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

    async fn output(&self, telnet: &mut Telnet) {
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

    async fn output(&self, telnet: &mut Telnet) {
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

    async fn output(&self, telnet: &mut Telnet) {
        telnet
            .print(&format!(
                "*** {}{} has entered Phoenix! [{}] ***\n",
                self.name.name,
                self.name.blurb,
                self.time.stamp()
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

    async fn output(&self, telnet: &mut Telnet) {
        telnet
            .print(&format!(
                "*** {}{} has left Phoenix! [{}] ***\n",
                self.name.name,
                self.name.blurb,
                self.time.stamp()
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

    async fn output(&self, telnet: &mut Telnet) {
        telnet
            .print(&format!(
                "*** {}{} has transferred to new connection. [{}] ***\n",
                self.name.name,
                self.name.blurb,
                self.time.stamp()
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

    async fn output(&self, telnet: &mut Telnet) {
        telnet
            .print(&format!(
                "*** {}{} is now attached. [{}] ***\n",
                self.name.name,
                self.name.blurb,
                self.time.stamp()
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

    async fn output(&self, telnet: &mut Telnet) {
        if self.intentional {
            telnet
                .print(&format!(
                    "*** {}{} has intentionally detached. [{}] ***\n",
                    self.name.name,
                    self.name.blurb,
                    self.time.stamp()
                ))
                .await;
        } else {
            telnet
                .print(&format!(
                    "*** {}{} has accidentally detached. [{}] ***\n",
                    self.name.name,
                    self.name.blurb,
                    self.time.stamp()
                ))
                .await;
        }
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

    async fn output(&self, telnet: &mut Telnet) {
        telnet
            .print(&format!(
                "*** {}{} is now here. [{}] ***\n",
                self.name.name,
                self.name.blurb,
                self.time.stamp()
            ))
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

    async fn output(&self, telnet: &mut Telnet) {
        telnet
            .print(&format!(
                "*** {}{} is now away. [{}] ***\n",
                self.name.name,
                self.name.blurb,
                self.time.stamp()
            ))
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

    async fn output(&self, telnet: &mut Telnet) {
        telnet
            .print(&format!(
                "*** {}{} is now busy. [{}] ***\n",
                self.name.name,
                self.name.blurb,
                self.time.stamp()
            ))
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

    async fn output(&self, telnet: &mut Telnet) {
        telnet
            .print(&format!(
                "*** {}{} is now gone. [{}] ***\n",
                self.name.name,
                self.name.blurb,
                self.time.stamp()
            ))
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

    async fn output(&self, telnet: &mut Telnet) {
        if self.is_public {
            telnet
                .print(&format!(
                    "*** {}{} has created discussion {}, \"{}\". [{}] ***\n",
                    self.creator.name,
                    self.creator.blurb,
                    self.discussion_name,
                    self.discussion_title,
                    self.time.stamp()
                ))
                .await;
        } else {
            telnet
                .print(&format!(
                    "*** {}{} has created private discussion {}. [{}] ***\n",
                    self.creator.name,
                    self.creator.blurb,
                    self.discussion_name,
                    self.time.stamp()
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

    async fn output(&self, telnet: &mut Telnet) {
        telnet
            .print(&format!(
                "*** {}{} has destroyed discussion {}. [{}] ***\n",
                self.name.name,
                self.name.blurb,
                self.discussion_name,
                self.time.stamp()
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

    async fn output(&self, telnet: &mut Telnet) {
        telnet
            .print(&format!(
                "*** {}{} has joined discussion {}. [{}] ***\n",
                self.name.name,
                self.name.blurb,
                self.discussion_name,
                self.time.stamp()
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

    async fn output(&self, telnet: &mut Telnet) {
        telnet
            .print(&format!(
                "*** {}{} has quit discussion {}. [{}] ***\n",
                self.name.name,
                self.name.blurb,
                self.discussion_name,
                self.time.stamp()
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

    async fn output(&self, telnet: &mut Telnet) {
        telnet
            .print(&format!(
                "*** {}{} has made discussion {} public. [{}] ***\n",
                self.name.name,
                self.name.blurb,
                self.discussion_name,
                self.time.stamp()
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

    async fn output(&self, telnet: &mut Telnet) {
        telnet
            .print(&format!(
                "*** {}{} has made discussion {} private. [{}] ***\n",
                self.name.name,
                self.name.blurb,
                self.discussion_name,
                self.time.stamp()
            ))
            .await;
    }
}

#[derive(Debug, Clone)]
pub struct PermitNotify {
    pub discussion_name: ArcStr,
    pub discussion_is_public: bool,
    pub name: Arc<Name>,
    pub is_explicit: bool,
    pub time: Timestamp,
}

impl PermitNotify {
    pub fn new(disc_name: ArcStr, disc_public: bool, who: Arc<Name>, flag: bool) -> Self {
        Self {
            discussion_name: disc_name,
            discussion_is_public: disc_public,
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

    async fn output(&self, telnet: &mut Telnet) {
        if self.discussion_is_public {
            if self.is_explicit {
                telnet
                    .print(&format!(
                        "*** {}{} has repermitted you to discussion {}. [{}] ***\n",
                        self.name.name,
                        self.name.blurb,
                        self.discussion_name,
                        self.time.stamp()
                    ))
                    .await;
            } else {
                telnet
                    .print(&format!(
                        "*** {}{} has explicitly permitted you to public discussion {}. [{}] ***\n",
                        self.name.name,
                        self.name.blurb,
                        self.discussion_name,
                        self.time.stamp()
                    ))
                    .await;
            }
        } else {
            if self.is_explicit {
                telnet
                    .print(&format!(
                        "*** {}{} has repermitted you to private discussion {}. [{}] ***\n",
                        self.name.name,
                        self.name.blurb,
                        self.discussion_name,
                        self.time.stamp()
                    ))
                    .await;
            } else {
                telnet
                    .print(&format!(
                        "*** {}{} has permitted you to private discussion {}. [{}] ***\n",
                        self.name.name,
                        self.name.blurb,
                        self.discussion_name,
                        self.time.stamp()
                    ))
                    .await;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct DepermitNotify {
    pub discussion_name: ArcStr,
    pub discussion_is_public: bool,
    pub name: Arc<Name>,
    pub is_explicit: bool,
    pub removed: Option<Arc<Name>>,
    pub time: Timestamp,
}

impl DepermitNotify {
    pub fn new(
        disc_name: ArcStr,
        disc_public: bool,
        who: Arc<Name>,
        flag: bool,
        removed_who: Option<Arc<Name>>,
    ) -> Self {
        Self {
            discussion_name: disc_name,
            discussion_is_public: disc_public,
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

    async fn output(&self, telnet: &mut Telnet) {
        let session_name = telnet.session_name().await;

        if self.discussion_is_public {
            if let Some(removed) = &self.removed {
                if removed.name.eq_ignore_ascii_case(&session_name) {
                    telnet.print(&format!("*** {}{} has depermitted and removed you from discussion {}. [{}] ***\n",
                        self.name.name, self.name.blurb, self.discussion_name, self.time.stamp())).await;
                } else {
                    telnet
                        .print(&format!(
                            "*** {}{} has been removed from discussion {}. [{}] ***\n",
                            removed.name,
                            removed.blurb,
                            self.discussion_name,
                            self.time.stamp()
                        ))
                        .await;
                }
            } else {
                telnet
                    .print(&format!(
                        "*** {}{} has depermitted you from discussion {}. [{}] ***\n",
                        self.name.name,
                        self.name.blurb,
                        self.discussion_name,
                        self.time.stamp()
                    ))
                    .await;
            }
        } else {
            if self.is_explicit {
                telnet.print(&format!("*** {}{} has explicitly depermitted you from private discussion {}. [{}] ***\n",
                    self.name.name, self.name.blurb, self.discussion_name, self.time.stamp())).await;
            } else {
                if let Some(removed) = &self.removed {
                    if removed.name.eq_ignore_ascii_case(&session_name) {
                        telnet.print(&format!("*** {}{} has depermitted and removed you from private discussion {}. [{}] ***\n",
                            self.name.name, self.name.blurb, self.discussion_name, self.time.stamp())).await;
                    } else {
                        telnet
                            .print(&format!(
                                "*** {}{} has been removed from discussion {}. [{}] ***\n",
                                removed.name,
                                removed.blurb,
                                self.discussion_name,
                                self.time.stamp()
                            ))
                            .await;
                    }
                } else {
                    telnet
                        .print(&format!(
                            "*** {}{} has depermitted you from private discussion {}. [{}] ***\n",
                            self.name.name,
                            self.name.blurb,
                            self.discussion_name,
                            self.time.stamp()
                        ))
                        .await;
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

    async fn output(&self, telnet: &mut Telnet) {
        let session_name = telnet.session_name().await;

        if self.appointee.name.eq_ignore_ascii_case(&session_name) {
            telnet
                .print(&format!(
                    "*** {}{} has appointed you as a moderator of discussion {}. [{}] ***\n",
                    self.appointer.name,
                    self.appointer.blurb,
                    self.discussion_name,
                    self.time.stamp()
                ))
                .await;
        } else {
            telnet
                .print(&format!(
                    "*** {}{} has appointed {}{} as a moderator of discussion {}. [{}] ***\n",
                    self.appointer.name,
                    self.appointer.blurb,
                    self.appointee.name,
                    self.appointee.blurb,
                    self.discussion_name,
                    self.time.stamp()
                ))
                .await;
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

    async fn output(&self, telnet: &mut Telnet) {
        let session_name = telnet.session_name().await;

        if self.unappointee.name.eq_ignore_ascii_case(&session_name) {
            telnet
                .print(&format!(
                    "*** {}{} has unappointed you as a moderator of discussion {}. [{}] ***\n",
                    self.unappointer.name,
                    self.unappointer.blurb,
                    self.discussion_name,
                    self.time.stamp()
                ))
                .await;
        } else {
            telnet
                .print(&format!(
                    "*** {}{} has unappointed {}{} as a moderator of discussion {}. [{}] ***\n",
                    self.unappointer.name,
                    self.unappointer.blurb,
                    self.unappointee.name,
                    self.unappointee.blurb,
                    self.discussion_name,
                    self.time.stamp()
                ))
                .await;
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

    async fn output(&self, telnet: &mut Telnet) {
        telnet
            .print(&format!(
                "*** {} has renamed to {}. [{}] ***\n",
                self.oldname,
                self.newname,
                self.time.stamp()
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

    pub async fn attach(&self, telnet: &mut Telnet) {
        self.sent.store(0, std::sync::atomic::Ordering::Relaxed);
        self.acknowledged
            .store(0, std::sync::atomic::Ordering::Relaxed);

        if telnet.acknowledge() {
            while self.send_next(telnet).await {}
        }
    }

    pub async fn enqueue(&self, telnet: Option<&mut Telnet>, out: Arc<dyn OutputObj>) {
        self.queue.lock().await.push(out);

        if let Some(telnet) = telnet {
            if telnet.acknowledge() {
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

    pub async fn send_next(&self, telnet: &mut Telnet) -> bool {
        let queue = self.queue.lock().await;
        let sent = self.sent.load(std::sync::atomic::Ordering::Relaxed);

        if sent >= queue.len() {
            drop(queue);
            telnet.redraw_input().await;
            false
        } else {
            let out = queue[sent].clone();
            drop(queue);

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
