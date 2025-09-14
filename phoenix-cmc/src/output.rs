use crate::name::Name;
use crate::sendlist::Sendlist;
use crate::telnet::Telnet;
use crate::text::Text;
use crate::timestamp::Timestamp;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use tokio::sync::Mutex;

// Output types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputType {
    UnknownOutput,
    TextOutput,
    PublicMessage,
    PrivateMessage,
    EntryOutput,
    ExitOutput,
    TransferOutput,
    AttachOutput,
    DetachOutput,
    HereOutput,
    AwayOutput,
    BusyOutput,
    GoneOutput,
    CreateOutput,
    DestroyOutput,
    JoinOutput,
    QuitOutput,
    PublicOutput,
    PrivateOutput,
    PermitOutput,
    DepermitOutput,
    AppointOutput,
    UnappointOutput,
    RenameOutput,
}

// Output classifications
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputClass {
    UnknownClass,
    TextClass,
    MessageClass,
    NotificationClass,
}

/// Unified output object enum that replaces the OutputObj trait
#[derive(Debug, Clone, PartialEq)]
pub enum Output {
    Text(TextOutput),
    Message(Message),
    EntryNotify(EntryNotify),
    ExitNotify(ExitNotify),
    TransferNotify(TransferNotify),
    AttachNotify(AttachNotify),
    DetachNotify(DetachNotify),
    HereNotify(HereNotify),
    AwayNotify(AwayNotify),
    BusyNotify(BusyNotify),
    GoneNotify(GoneNotify),
    CreateNotify(CreateNotify),
    DestroyNotify(DestroyNotify),
    JoinNotify(JoinNotify),
    QuitNotify(QuitNotify),
    PublicNotify(PublicNotify),
    PrivateNotify(PrivateNotify),
    PermitNotify(PermitNotify),
    DepermitNotify(DepermitNotify),
    AppointNotify(AppointNotify),
    UnappointNotify(UnappointNotify),
    RenameNotify(RenameNotify),
}

impl Output {
    pub fn output_type(&self) -> OutputType {
        match self {
            Output::Text(obj) => obj.output_type(),
            Output::Message(obj) => obj.output_type(),
            Output::EntryNotify(obj) => obj.output_type(),
            Output::ExitNotify(obj) => obj.output_type(),
            Output::TransferNotify(obj) => obj.output_type(),
            Output::AttachNotify(obj) => obj.output_type(),
            Output::DetachNotify(obj) => obj.output_type(),
            Output::HereNotify(obj) => obj.output_type(),
            Output::AwayNotify(obj) => obj.output_type(),
            Output::BusyNotify(obj) => obj.output_type(),
            Output::GoneNotify(obj) => obj.output_type(),
            Output::CreateNotify(obj) => obj.output_type(),
            Output::DestroyNotify(obj) => obj.output_type(),
            Output::JoinNotify(obj) => obj.output_type(),
            Output::QuitNotify(obj) => obj.output_type(),
            Output::PublicNotify(obj) => obj.output_type(),
            Output::PrivateNotify(obj) => obj.output_type(),
            Output::PermitNotify(obj) => obj.output_type(),
            Output::DepermitNotify(obj) => obj.output_type(),
            Output::AppointNotify(obj) => obj.output_type(),
            Output::UnappointNotify(obj) => obj.output_type(),
            Output::RenameNotify(obj) => obj.output_type(),
        }
    }

    pub fn output_class(&self) -> OutputClass {
        match self {
            Output::Text(obj) => obj.output_class(),
            Output::Message(obj) => obj.output_class(),
            Output::EntryNotify(obj) => obj.output_class(),
            Output::ExitNotify(obj) => obj.output_class(),
            Output::TransferNotify(obj) => obj.output_class(),
            Output::AttachNotify(obj) => obj.output_class(),
            Output::DetachNotify(obj) => obj.output_class(),
            Output::HereNotify(obj) => obj.output_class(),
            Output::AwayNotify(obj) => obj.output_class(),
            Output::BusyNotify(obj) => obj.output_class(),
            Output::GoneNotify(obj) => obj.output_class(),
            Output::CreateNotify(obj) => obj.output_class(),
            Output::DestroyNotify(obj) => obj.output_class(),
            Output::JoinNotify(obj) => obj.output_class(),
            Output::QuitNotify(obj) => obj.output_class(),
            Output::PublicNotify(obj) => obj.output_class(),
            Output::PrivateNotify(obj) => obj.output_class(),
            Output::PermitNotify(obj) => obj.output_class(),
            Output::DepermitNotify(obj) => obj.output_class(),
            Output::AppointNotify(obj) => obj.output_class(),
            Output::UnappointNotify(obj) => obj.output_class(),
            Output::RenameNotify(obj) => obj.output_class(),
        }
    }

    pub fn time(&self) -> Timestamp {
        match self {
            Output::Text(obj) => obj.time(),
            Output::Message(obj) => obj.time(),
            Output::EntryNotify(obj) => obj.time(),
            Output::ExitNotify(obj) => obj.time(),
            Output::TransferNotify(obj) => obj.time(),
            Output::AttachNotify(obj) => obj.time(),
            Output::DetachNotify(obj) => obj.time(),
            Output::HereNotify(obj) => obj.time(),
            Output::AwayNotify(obj) => obj.time(),
            Output::BusyNotify(obj) => obj.time(),
            Output::GoneNotify(obj) => obj.time(),
            Output::CreateNotify(obj) => obj.time(),
            Output::DestroyNotify(obj) => obj.time(),
            Output::JoinNotify(obj) => obj.time(),
            Output::QuitNotify(obj) => obj.time(),
            Output::PublicNotify(obj) => obj.time(),
            Output::PrivateNotify(obj) => obj.time(),
            Output::PermitNotify(obj) => obj.time(),
            Output::DepermitNotify(obj) => obj.time(),
            Output::AppointNotify(obj) => obj.time(),
            Output::UnappointNotify(obj) => obj.time(),
            Output::RenameNotify(obj) => obj.time(),
        }
    }

    pub async fn output(&self, telnet: &Telnet) {
        match self {
            Output::Text(obj) => obj.output(telnet).await,
            Output::Message(obj) => obj.output(telnet).await,
            Output::EntryNotify(obj) => obj.output(telnet).await,
            Output::ExitNotify(obj) => obj.output(telnet).await,
            Output::TransferNotify(obj) => obj.output(telnet).await,
            Output::AttachNotify(obj) => obj.output(telnet).await,
            Output::DetachNotify(obj) => obj.output(telnet).await,
            Output::HereNotify(obj) => obj.output(telnet).await,
            Output::AwayNotify(obj) => obj.output(telnet).await,
            Output::BusyNotify(obj) => obj.output(telnet).await,
            Output::GoneNotify(obj) => obj.output(telnet).await,
            Output::CreateNotify(obj) => obj.output(telnet).await,
            Output::DestroyNotify(obj) => obj.output(telnet).await,
            Output::JoinNotify(obj) => obj.output(telnet).await,
            Output::QuitNotify(obj) => obj.output(telnet).await,
            Output::PublicNotify(obj) => obj.output(telnet).await,
            Output::PrivateNotify(obj) => obj.output(telnet).await,
            Output::PermitNotify(obj) => obj.output(telnet).await,
            Output::DepermitNotify(obj) => obj.output(telnet).await,
            Output::AppointNotify(obj) => obj.output(telnet).await,
            Output::UnappointNotify(obj) => obj.output(telnet).await,
            Output::RenameNotify(obj) => obj.output(telnet).await,
        }
    }
}

// From implementations for easy conversion
impl From<TextOutput> for Output {
    fn from(obj: TextOutput) -> Self {
        Output::Text(obj)
    }
}

impl From<Message> for Output {
    fn from(obj: Message) -> Self {
        Output::Message(obj)
    }
}

impl From<EntryNotify> for Output {
    fn from(obj: EntryNotify) -> Self {
        Output::EntryNotify(obj)
    }
}

impl From<ExitNotify> for Output {
    fn from(obj: ExitNotify) -> Self {
        Output::ExitNotify(obj)
    }
}

impl From<TransferNotify> for Output {
    fn from(obj: TransferNotify) -> Self {
        Output::TransferNotify(obj)
    }
}

impl From<AttachNotify> for Output {
    fn from(obj: AttachNotify) -> Self {
        Output::AttachNotify(obj)
    }
}

impl From<DetachNotify> for Output {
    fn from(obj: DetachNotify) -> Self {
        Output::DetachNotify(obj)
    }
}

impl From<HereNotify> for Output {
    fn from(obj: HereNotify) -> Self {
        Output::HereNotify(obj)
    }
}

impl From<AwayNotify> for Output {
    fn from(obj: AwayNotify) -> Self {
        Output::AwayNotify(obj)
    }
}

impl From<BusyNotify> for Output {
    fn from(obj: BusyNotify) -> Self {
        Output::BusyNotify(obj)
    }
}

impl From<GoneNotify> for Output {
    fn from(obj: GoneNotify) -> Self {
        Output::GoneNotify(obj)
    }
}

impl From<CreateNotify> for Output {
    fn from(obj: CreateNotify) -> Self {
        Output::CreateNotify(obj)
    }
}

impl From<DestroyNotify> for Output {
    fn from(obj: DestroyNotify) -> Self {
        Output::DestroyNotify(obj)
    }
}

impl From<JoinNotify> for Output {
    fn from(obj: JoinNotify) -> Self {
        Output::JoinNotify(obj)
    }
}

impl From<QuitNotify> for Output {
    fn from(obj: QuitNotify) -> Self {
        Output::QuitNotify(obj)
    }
}

impl From<PublicNotify> for Output {
    fn from(obj: PublicNotify) -> Self {
        Output::PublicNotify(obj)
    }
}

impl From<PrivateNotify> for Output {
    fn from(obj: PrivateNotify) -> Self {
        Output::PrivateNotify(obj)
    }
}

impl From<PermitNotify> for Output {
    fn from(obj: PermitNotify) -> Self {
        Output::PermitNotify(obj)
    }
}

impl From<DepermitNotify> for Output {
    fn from(obj: DepermitNotify) -> Self {
        Output::DepermitNotify(obj)
    }
}

impl From<AppointNotify> for Output {
    fn from(obj: AppointNotify) -> Self {
        Output::AppointNotify(obj)
    }
}

impl From<UnappointNotify> for Output {
    fn from(obj: UnappointNotify) -> Self {
        Output::UnappointNotify(obj)
    }
}

impl From<RenameNotify> for Output {
    fn from(obj: RenameNotify) -> Self {
        Output::RenameNotify(obj)
    }
}

#[derive(Debug, Clone)]
pub struct TextOutput {
    pub text: Text,
    pub time: Timestamp,
}

impl TextOutput {
    pub fn new(text: impl Into<Text>) -> Output {
        Output::Text(Self { text: text.into(), time: Timestamp::new() })
    }
}

#[async_trait]
impl OutputObj for TextOutput {
    fn output_type(&self) -> OutputType {
        OutputType::TextOutput
    }

    fn output_class(&self) -> OutputClass {
        OutputClass::TextClass
    }

    fn time(&self) -> Timestamp {
        self.time
    }

    async fn output(&self, telnet: &Telnet) {
        telnet.output(&self.text).await;
    }
}

/// Message handle.
#[derive(Debug, Clone)]
pub struct Message(pub Arc<MessageInner>);

#[derive(Debug)]
pub struct MessageInner {
    pub output_type: OutputType,
    pub from: Name,
    pub to: Arc<Sendlist>,
    pub text: Text,
    pub time: Timestamp,
}

impl Message {
    pub fn new(output_type: OutputType, sender: Name, dest: Arc<Sendlist>, msg: impl Into<Text>) -> Output {
        let inner = MessageInner { output_type, from: sender, to: dest, text: msg.into(), time: Timestamp::new() };
        Output::Message(Self(Arc::new(inner)))
    }
}

#[async_trait]
impl OutputObj for Message {
    fn output_type(&self) -> OutputType {
        self.0.output_type
    }

    fn output_class(&self) -> OutputClass {
        OutputClass::MessageClass
    }

    fn time(&self) -> Timestamp {
        self.0.time
    }

    async fn output(&self, telnet: &Telnet) {
        telnet.print_message(self.0.output_type, self.0.time, &self.0.from, &self.0.to, &self.0.text).await;
    }
}

#[derive(Debug, Clone)]
pub struct EntryNotify {
    pub name: Name,
    pub time: Timestamp,
}

impl EntryNotify {
    pub fn new(who: Name) -> Output {
        Output::EntryNotify(Self { name: who, time: Timestamp::new() })
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

    async fn output(&self, telnet: &Telnet) {
        let name = &self.name;
        let stamp = &self.time.stamp();
        telnet.output(&format!("*** {name} has entered Phoenix! [{stamp}] ***\n")).await;
    }
}

#[derive(Debug, Clone)]
pub struct ExitNotify {
    pub name: Name,
    pub time: Timestamp,
}

impl ExitNotify {
    pub fn new(who: Name) -> Output {
        Output::ExitNotify(Self { name: who, time: Timestamp::new() })
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

    async fn output(&self, telnet: &Telnet) {
        let name = &self.name;
        let stamp = &self.time.stamp();
        telnet.output(&format!("*** {name} has left Phoenix! [{stamp}] ***\n")).await;
    }
}

#[derive(Debug, Clone)]
pub struct TransferNotify {
    pub name: Name,
    pub time: Timestamp,
}

impl TransferNotify {
    pub fn new(who: Name) -> Output {
        Output::TransferNotify(Self { name: who, time: Timestamp::new() })
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

    async fn output(&self, telnet: &Telnet) {
        let name = &self.name;
        let stamp = &self.time.stamp();
        telnet.output(&format!("*** {name} has transferred to new connection. [{stamp}] ***\n")).await;
    }
}

#[derive(Debug, Clone)]
pub struct AttachNotify {
    pub name: Name,
    pub time: Timestamp,
}

impl AttachNotify {
    pub fn new(who: Name) -> Output {
        Output::AttachNotify(Self { name: who, time: Timestamp::new() })
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

    async fn output(&self, telnet: &Telnet) {
        let name = &self.name;
        let stamp = &self.time.stamp();
        telnet.output(&format!("*** {name} is now attached. [{stamp}] ***\n")).await;
    }
}

#[derive(Debug, Clone)]
pub struct DetachNotify {
    pub name: Name,
    pub intentional: bool,
    pub time: Timestamp,
}

impl DetachNotify {
    pub fn new(who: Name, i: bool) -> Output {
        Output::DetachNotify(Self { name: who, intentional: i, time: Timestamp::new() })
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

    async fn output(&self, telnet: &Telnet) {
        let name = &self.name;
        let stamp = &self.time.stamp();
        let intentionally = if self.intentional { "intentionally" } else { "accidentally" };
        telnet.output(&format!("*** {name} has {intentionally} detached. [{stamp}] ***\n")).await;
    }
}

#[derive(Debug, Clone)]
pub struct HereNotify {
    pub name: Name,
    pub time: Timestamp,
}

impl HereNotify {
    pub fn new(who: Name) -> Output {
        Output::HereNotify(Self { name: who, time: Timestamp::new() })
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

    async fn output(&self, telnet: &Telnet) {
        let name = &self.name;
        let stamp = &self.time.stamp();
        telnet.output(&format!("*** {name} is now here. [{stamp}] ***\n")).await;
    }
}

#[derive(Debug, Clone)]
pub struct AwayNotify {
    pub name: Name,
    pub time: Timestamp,
}

impl AwayNotify {
    pub fn new(who: Name) -> Output {
        Output::AwayNotify(Self { name: who, time: Timestamp::new() })
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

    async fn output(&self, telnet: &Telnet) {
        let name = &self.name;
        let stamp = &self.time.stamp();
        telnet.output(&format!("*** {name} is now away. [{stamp}] ***\n")).await;
    }
}

#[derive(Debug, Clone)]
pub struct BusyNotify {
    pub name: Name,
    pub time: Timestamp,
}

impl BusyNotify {
    pub fn new(who: Name) -> Output {
        Output::BusyNotify(Self { name: who, time: Timestamp::new() })
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

    async fn output(&self, telnet: &Telnet) {
        let name = &self.name;
        let stamp = &self.time.stamp();
        telnet.output(&format!("*** {name} is now busy. [{stamp}] ***\n")).await;
    }
}

#[derive(Debug, Clone)]
pub struct GoneNotify {
    pub name: Name,
    pub time: Timestamp,
}

impl GoneNotify {
    pub fn new(who: Name) -> Output {
        Output::GoneNotify(Self { name: who, time: Timestamp::new() })
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

    async fn output(&self, telnet: &Telnet) {
        let name = &self.name;
        let stamp = &self.time.stamp();
        telnet.output(&format!("*** {name} is now gone. [{stamp}] ***\n")).await;
    }
}

#[derive(Debug, Clone)]
pub struct CreateNotify {
    pub discussion_name: Text,
    pub discussion_title: Text,
    pub is_public: bool,
    pub creator: Name,
    pub time: Timestamp,
}

impl CreateNotify {
    pub fn new(disc_name: Text, disc_title: Text, is_public: bool, creator: Name) -> Output {
        Output::CreateNotify(Self { discussion_name: disc_name, discussion_title: disc_title, is_public, creator, time: Timestamp::new() })
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

    async fn output(&self, telnet: &Telnet) {
        let creator = &self.creator;
        let disc = &self.discussion_name;
        let title = &self.discussion_title;
        let stamp = &self.time.stamp();
        if self.is_public {
            telnet.output(&format!("*** {creator} has created discussion {disc}, \"{title}\". [{stamp}] ***\n")).await;
        } else {
            telnet.output(&format!("*** {creator} has created private discussion {disc}. [{stamp}] ***\n")).await;
        }
    }
}

#[derive(Debug, Clone)]
pub struct DestroyNotify {
    pub discussion_name: Text,
    pub name: Name,
    pub time: Timestamp,
}

impl DestroyNotify {
    pub fn new(disc_name: Text, who: Name) -> Output {
        Output::DestroyNotify(Self { discussion_name: disc_name, name: who, time: Timestamp::new() })
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

    async fn output(&self, telnet: &Telnet) {
        let name = &self.name;
        let disc = &self.discussion_name;
        let stamp = &self.time.stamp();
        telnet.output(&format!("*** {name} has destroyed discussion {disc}. [{stamp}] ***\n")).await;
    }
}

#[derive(Debug, Clone)]
pub struct JoinNotify {
    pub discussion_name: Text,
    pub name: Name,
    pub time: Timestamp,
}

impl JoinNotify {
    pub fn new(disc_name: Text, who: Name) -> Output {
        Output::JoinNotify(Self { discussion_name: disc_name, name: who, time: Timestamp::new() })
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

    async fn output(&self, telnet: &Telnet) {
        let name = &self.name;
        let disc = &self.discussion_name;
        let stamp = &self.time.stamp();
        telnet.output(&format!("*** {name} has joined discussion {disc}. [{stamp}] ***\n")).await;
    }
}

#[derive(Debug, Clone)]
pub struct QuitNotify {
    pub discussion_name: Text,
    pub name: Name,
    pub time: Timestamp,
}

impl QuitNotify {
    pub fn new(disc_name: Text, who: Name) -> Output {
        Output::QuitNotify(Self { discussion_name: disc_name, name: who, time: Timestamp::new() })
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

    async fn output(&self, telnet: &Telnet) {
        let name = &self.name;
        let disc = &self.discussion_name;
        let stamp = &self.time.stamp();
        telnet.output(&format!("*** {name} has quit discussion {disc}. [{stamp}] ***\n")).await;
    }
}

#[derive(Debug, Clone)]
pub struct PublicNotify {
    pub discussion_name: Text,
    pub name: Name,
    pub time: Timestamp,
}

impl PublicNotify {
    pub fn new(disc_name: Text, who: Name) -> Output {
        Output::PublicNotify(Self { discussion_name: disc_name, name: who, time: Timestamp::new() })
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

    async fn output(&self, telnet: &Telnet) {
        let name = &self.name;
        let disc = &self.discussion_name;
        let stamp = &self.time.stamp();
        telnet.output(&format!("*** {name} has made discussion {disc} public. [{stamp}] ***\n")).await;
    }
}

#[derive(Debug, Clone)]
pub struct PrivateNotify {
    pub discussion_name: Text,
    pub name: Name,
    pub time: Timestamp,
}

impl PrivateNotify {
    pub fn new(disc_name: Text, who: Name) -> Output {
        Output::PrivateNotify(Self { discussion_name: disc_name, name: who, time: Timestamp::new() })
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

    async fn output(&self, telnet: &Telnet) {
        let name = &self.name;
        let disc = &self.discussion_name;
        let stamp = &self.time.stamp();
        telnet.output(&format!("*** {name} has made discussion {disc} private. [{stamp}] ***\n")).await;
    }
}

#[derive(Debug, Clone)]
pub struct PermitNotify {
    pub discussion_name: Text,
    pub is_public: bool,
    pub name: Name,
    pub is_explicit: bool,
    pub time: Timestamp,
}

impl PermitNotify {
    pub fn new(disc_name: Text, public: bool, who: Name, flag: bool) -> Output {
        Output::PermitNotify(Self { discussion_name: disc_name, is_public: public, name: who, is_explicit: flag, time: Timestamp::new() })
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

    async fn output(&self, telnet: &Telnet) {
        let name = &self.name;
        let disc = &self.discussion_name;
        let stamp = &self.time.stamp();
        if self.is_public {
            if self.is_explicit {
                telnet.output(&format!("*** {name} has repermitted you to discussion {disc}. [{stamp}] ***\n")).await;
            } else {
                telnet.output(&format!("*** {name} has explicitly permitted you to public discussion {disc}. [{stamp}] ***\n")).await;
            }
        } else {
            if self.is_explicit {
                telnet.output(&format!("*** {name} has repermitted you to private discussion {disc}. [{stamp}] ***\n")).await;
            } else {
                telnet.output(&format!("*** {name} has permitted you to private discussion {disc}. [{stamp}] ***\n")).await;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct DepermitNotify {
    pub discussion_name: Text,
    pub is_public: bool,
    pub name: Name,
    pub is_explicit: bool,
    pub removed: Option<Name>,
    pub time: Timestamp,
}

impl DepermitNotify {
    pub fn new(disc_name: Text, public: bool, who: Name, flag: bool, removed_who: Option<Name>) -> Output {
        Output::DepermitNotify(Self {
            discussion_name: disc_name,
            is_public: public,
            name: who,
            is_explicit: flag,
            removed: removed_who,
            time: Timestamp::new(),
        })
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

    async fn output(&self, telnet: &Telnet) {
        let session_name = telnet.session_name();
        let name = &self.name;
        let disc = &self.discussion_name;
        let stamp = &self.time.stamp();

        if self.is_public {
            if let Some(removed) = &self.removed {
                if let Some(session_name) = &session_name {
                    let session_name = Name::new(session_name.as_str(), None::<String>);
                    if removed == &session_name {
                        telnet.output(&format!("*** {name} has depermitted and removed you from discussion {disc}. [{stamp}] ***\n")).await;
                    } else {
                        telnet.output(&format!("*** {removed} has been removed from discussion {disc}. [{stamp}] ***\n")).await;
                    }
                } else {
                    telnet.output(&format!("*** {removed} has been removed from discussion {disc}. [{stamp}] ***\n")).await;
                }
            } else {
                telnet.output(&format!("*** {name} has depermitted you from discussion {disc}. [{stamp}] ***\n")).await;
            }
        } else {
            if self.is_explicit {
                telnet.output(&format!("*** {name} has explicitly depermitted you from private discussion {disc}. [{stamp}] ***\n")).await;
            } else {
                if let Some(removed) = &self.removed {
                    if let Some(session_name) = &session_name {
                        let session_name = Name::new(session_name.as_str(), None::<String>);
                        if removed == &session_name {
                            telnet.output(&format!("*** {name} has depermitted and removed you from private discussion {disc}. [{stamp}] ***\n")).await;
                        } else {
                            telnet.output(&format!("*** {removed} has been removed from discussion {disc}. [{stamp}] ***\n")).await;
                        }
                    } else {
                        telnet.output(&format!("*** {removed} has been removed from discussion {disc}. [{stamp}] ***\n")).await;
                    }
                } else {
                    telnet.output(&format!("*** {name} has depermitted you from private discussion {disc}. [{stamp}] ***\n")).await;
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppointNotify {
    pub discussion_name: Text,
    pub appointer: Name,
    pub appointee: Name,
    pub time: Timestamp,
}

impl AppointNotify {
    pub fn new(disc_name: Text, who1: Name, who2: Name) -> Self {
        Self { discussion_name: disc_name, appointer: who1, appointee: who2, time: Timestamp::new() }
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

    async fn output(&self, telnet: &Telnet) {
        let session_name = telnet.session_name();
        let appointer = &self.appointer;
        let appointee = if let Some(session_name) = &session_name {
            let session_name = Name::new(session_name.as_str(), None::<String>);
            self.appointee.you(&session_name)
        } else {
            self.appointee.as_str()
        };
        let disc = &self.discussion_name;
        let stamp = &self.time.stamp();

        telnet.output(&format!("*** {appointer} has appointed {appointee} as a moderator of discussion {disc}. [{stamp}] ***\n")).await;
    }
}

#[derive(Debug, Clone)]
pub struct UnappointNotify {
    pub discussion_name: Text,
    pub unappointer: Name,
    pub unappointee: Name,
    pub time: Timestamp,
}

impl UnappointNotify {
    pub fn new(disc_name: Text, who1: Name, who2: Name) -> Self {
        Self { discussion_name: disc_name, unappointer: who1, unappointee: who2, time: Timestamp::new() }
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

    async fn output(&self, telnet: &Telnet) {
        let session_name = telnet.session_name();
        let unappointer = &self.unappointer;
        let unappointee = if let Some(session_name) = &session_name {
            let session_name = Name::new(session_name.as_str(), None::<String>);
            self.unappointee.you(&session_name)
        } else {
            self.unappointee.as_str()
        };
        let disc = &self.discussion_name;
        let stamp = &self.time.stamp();

        telnet.output(&format!("*** {unappointer} has unappointed {unappointee} as a moderator of discussion {disc}. [{stamp}] ***\n")).await;
    }
}

#[derive(Debug, Clone)]
pub struct RenameNotify {
    pub oldname: Text,
    pub newname: Text,
    pub time: Timestamp,
}

impl RenameNotify {
    pub fn new(oldstr: impl Into<Text>, newstr: impl Into<Text>) -> Output {
        Output::RenameNotify(Self { oldname: oldstr.into(), newname: newstr.into(), time: Timestamp::new() })
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

    async fn output(&self, telnet: &Telnet) {
        let oldname = &self.oldname;
        let newname = &self.newname;
        let stamp = &self.time.stamp();
        telnet.output(&format!("*** {oldname} has renamed to {newname}. [{stamp}] ***\n")).await;
    }
}

// Output stream for queuing output objects
#[derive(Debug)]
pub struct OutputStream {
    pub queue: tokio::sync::Mutex<Vec<Output>>,
    pub acknowledged: std::sync::atomic::AtomicUsize,
    pub sent: std::sync::atomic::AtomicUsize,
}

impl OutputStream {
    pub fn new() -> Self {
        Self { queue: tokio::sync::Mutex::new(Vec::new()), acknowledged: std::sync::atomic::AtomicUsize::new(0), sent: std::sync::atomic::AtomicUsize::new(0) }
    }

    pub async fn acknowledge(&self) {
        let sent = self.sent.load(std::sync::atomic::Ordering::Relaxed);
        let ack = self.acknowledged.load(std::sync::atomic::Ordering::Relaxed);
        if ack < sent {
            self.acknowledged.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
    }

    pub async fn attach(&self, telnet: &Telnet) {
        self.sent.store(0, std::sync::atomic::Ordering::Relaxed);
        self.acknowledged.store(0, std::sync::atomic::Ordering::Relaxed);

        if telnet.acknowledge() {
            while self.send_next(telnet).await {}
        }
    }

    pub async fn enqueue(&self, telnet: Option<&Telnet>, out: Output) {
        self.queue.lock().await.push(out);

        if let Some(telnet) = telnet {
            if telnet.acknowledge() {
                while self.send_next(telnet).await {}
            } else if self.queue.lock().await.len() == 1 {
                self.send_next(telnet).await;
            }
        }
    }

    pub async fn unenqueue(&self, out: &Output) {
        let mut queue = self.queue.lock().await;
        queue.retain(|item| item != out);
    }

    pub async fn dequeue(&self) {
        let ack = self.acknowledged.load(std::sync::atomic::Ordering::Relaxed);
        if ack > 0 {
            let mut queue = self.queue.lock().await;
            let to_remove = ack.min(queue.len());
            queue.drain(..to_remove);
            self.acknowledged.fetch_sub(to_remove, std::sync::atomic::Ordering::Relaxed);
            self.sent.fetch_sub(to_remove, std::sync::atomic::Ordering::Relaxed);
        }
    }

    pub async fn send_next(&self, telnet: &Telnet) -> bool {
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
