#![no_std]
use gstd::{msg, prelude::*, ActorId};
use launch_io::*;

pub const MAX_VALUE: u64 = 10_000;

#[derive(Default, Encode, Decode, TypeInfo)]
pub struct CurrentSession {
    pub altitude: u32,
    pub weather: u32,
    pub fuel_price: u32,
    pub payload_price: u32,
}

#[derive(Default, Encode, Decode, TypeInfo)]
pub struct Participant {
    pub name: String,
    pub balance: u32,
}

#[derive(Default, Encode, Decode, TypeInfo)]
pub struct LaunchSite {
    pub name: String,
    pub owner: ActorId,
    pub participants: BTreeMap<ActorId, Participant>,
    pub current_session: Option<CurrentSession>,
}

static mut LAUNCH_SITE: Option<LaunchSite> = None;

impl LaunchSite {
    fn info(&self) {
        msg::reply(
            Event::Info {
                name: self.name.clone(),
                owner: self.owner,
                has_current_session: self.current_session.is_some(),
            },
            0,
        )
        .expect("Error in a reply `LaunchSite::info");
    }

    fn new_participant(&self, name: String) {
        let actor_id = msg::source();

        unimplemented!()
    }

    fn rename_participant(&self, name: String) {
        let actor_id = msg::source();

        unimplemented!()
    }

    fn new_session(&self) {

        unimplemented!()
    }

    fn register_on_launch(&self, fuel_amount: u32, payload_amount: u32) {

        unimplemented!()
    }
}

#[gstd::async_main]
async fn main() {
    let action: Action = msg::load().expect("Unable to decode `Action`");
    let launch_site = unsafe { LAUNCH_SITE.get_or_insert(Default::default()) };
    match action {
        Action::Info => { launch_site.info(); },
        Action::RegisterParticipant(name) => { launch_site.new_participant(name); },
        Action::ChangeParticipantName(name) => { launch_site.rename_participant(name); },
        Action::StartNewSession => { launch_site.new_session(); },
        Action::RegisterOnLaunch { fuel_amount, payload_amount } => { launch_site.register_on_launch(fuel_amount, payload_amount); },
    }
}

#[no_mangle]
unsafe extern "C" fn init() {
    let name: String = String::from_utf8(msg::load_bytes().expect("Cant load init message"))
        .expect("Error in decoding");
    let launch_site = LaunchSite {
        name,
        owner: msg::source(),
        .. Default::default()
    };
    LAUNCH_SITE = Some(launch_site);
}

#[no_mangle]
extern "C" fn state() {
    let launch_site = unsafe { LAUNCH_SITE.get_or_insert(Default::default()) };
    msg::reply(launch_site, 0).expect("Failed to share state");
}

#[no_mangle]
extern "C" fn metahash() {
    let metahash: [u8; 32] = include!("../.metahash");
    msg::reply(metahash, 0).expect("Failed to share metahash");
}
