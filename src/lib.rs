#![no_std]
use gstd::{msg, prelude::*, ActorId};
use launch_io::*;

pub const MAX_VALUE: u64 = 10_000;

#[derive(Default, Encode, Decode, TypeInfo)]
pub struct SessionStrategy {
    pub fuel: u32,
    pub payload: u32,
}

#[derive(Default, Encode, Decode, TypeInfo)]
pub struct CurrentSession {
    pub altitude: u32,
    pub weather: u32,
    pub fuel_price: u32,
    pub payload_value: u32,
    pub registered: BTreeMap<ActorId, SessionStrategy>,
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
        .expect("Error in a reply `::info");
    }

    fn new_participant(&mut self, name: String) {
        let actor_id = msg::source();

        if self.participants.contains_key(&actor_id) {
            panic!("There is already participant registered with this id");
        }

        self.participants.insert(
            actor_id,
            Participant {
                name: name.clone(),
                balance: 0,
            }
        );

        msg::reply(
            Event::NewParticipant { id: actor_id, name },
            0,
        ).expect("failed to reply in ::new_participant");
    }

    fn rename_participant(&mut self, name: String) {
        let actor_id = msg::source();

        if !self.participants.contains_key(&actor_id) {
            panic!("There is no participant registered with this id");
        }

        let participant = self.participants.get_mut(&actor_id).expect("checked above that exists");

        participant.name = name.clone();

        msg::reply(
            Event::ParticipantNameChange { id: actor_id, name },
            0,
        ).expect("failed to reply in ::rename_participant");
    }

    fn new_session(&mut self) {

        let actor_id = msg::source();

        assert_eq!(actor_id, self.owner);
        assert!(self.current_session.is_none());

        let random_weather = 100;
        let random_fuel_price = 100;
        let random_payload_value = 100;
        let random_altitude = 120_000;

        self.current_session = Some(CurrentSession {
            weather: random_weather,
            fuel_price: random_fuel_price,
            payload_value: random_payload_value,
            altitude: random_altitude,
            registered: Default::default(),
        });

        msg::reply(
            Event::NewLaunch {
                id: 0,
                name: "Unnamed".to_string(),
                weather: random_weather,
                fuel_price: random_fuel_price,
                payload_value: random_payload_value,
                altitude: random_altitude,
            },
            0,
        ).expect("failed to reply in ::new_session");
    }

    fn register_on_launch(&mut self, fuel_amount: u32, payload_amount: u32) {
        let actor_id = msg::source();

        assert!(self.current_session.is_some());

        let current_session = self.current_session.as_mut().expect("checked above that exists");

        if current_session.registered.contains_key(&actor_id) {
            // already registered

            panic!("Participant already registered on the session");
        }

        current_session.registered.insert(actor_id, SessionStrategy { fuel: fuel_amount, payload: payload_amount });

        msg::reply(
            Event::LaunchRegistration {
                id: 0,
                participant: actor_id,
            },
            0,
        ).expect("failed to reply in ::new_session");
    }

    fn execute_session(&mut self) {

        struct CurrentStat {
            alive: bool,
            fuel_left: u32,
            last_altitude: u32,
            payload: u32,
        }

        let session_data = self.current_session.take().expect("There should be active session to execute");

        let mut current_altitude = 0;
        let total_rounds = 3;
        let _weather = session_data.weather;

        let mut current_stats: BTreeMap<ActorId, CurrentStat> = BTreeMap::new();

        for (id, strategy) in session_data.registered.iter() {
            current_stats.insert(*id, CurrentStat { alive: true, fuel_left: strategy.fuel, last_altitude: 0, payload: strategy.payload } );
        }

        for _ in 0..total_rounds {
            current_altitude += session_data.altitude / total_rounds;

            for (id, strategy) in session_data.registered.iter() {
                let fuel_burn = strategy.payload / total_rounds;

                let current_stat = current_stats.get_mut(&id).expect("all have stats");

                if !current_stat.alive { continue; } // already failed;

                if current_stat.fuel_left < fuel_burn {
                    // fuel is over
                    current_stat.alive = false;
                } else {
                    current_stat.last_altitude = current_altitude;
                }

                // weather random affect?
            }
        }

        let mut outcome_participants = vec![];
        for (id, stat) in current_stats.iter() {
            let earnings = stat.payload * session_data.payload_value;

            outcome_participants.push(
                (
                    *id,
                    stat.alive,
                    stat.last_altitude,
                    earnings,
                )
            );

            let leaderboard_entry = self.participants.get_mut(&id).expect("Should have existed in leaderboards");

            leaderboard_entry.balance += earnings;
        }

        // handle round results

        msg::reply(
            Event::LaunchFinished {
                id: 0,
                stats: outcome_participants,
            },
            0,
        ).expect("failed to reply in ::new_session");
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
        Action::ExecuteSession => { launch_site.execute_session(); },
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
