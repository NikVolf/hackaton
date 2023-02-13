#![no_std]

use codec::{Decode, Encode};
use gmeta::{InOut, Metadata};
use gstd::{prelude::*, ActorId};

pub struct ProgramMetadata;

impl Metadata for ProgramMetadata {
    type Init = InOut<String, ()>;
    type Handle = InOut<Action, Event>;
    type Reply = InOut<(), ()>;
    type Others = InOut<(), ()>;
    type Signal = ();
    type State = LaunchSiteState;
}

#[derive(Encode, Decode, TypeInfo, Debug)]
pub enum Action {
    Info,
    RegisterParticipant(String),
    ChangeParticipantName(String),
    StartNewSession,
    RegisterOnLaunch { fuel_amount: u32, payload_amount: u32 },
    ExecuteSession,
}

#[derive(Encode, Debug, PartialEq, Eq, Decode, TypeInfo)]
pub enum Event {
    Info { owner: ActorId, name: String, has_current_session: bool },
    NewParticipant { id: ActorId, name: String },
    ParticipantNameChange { id: ActorId, name: String },
    NewLaunch { id: u32, name: String, weather: u32, altitude: u32, fuel_price: u32, payload_value: u32 },
    LaunchRegistration { id: u32, participant: ActorId },
    LaunchStarted { id: u32 },
    LaunchFinished { id: u32 },
    SessionInfo { weather: u32, altitude: u32, fuel_price: u32, payload_value: u32 },
    NoCurrentSession,
}

#[derive(Default, Encode, Decode, TypeInfo)]
pub struct CurrentSesionInfo {
    pub name: String,
    pub weather: u32,
    pub altitude: u32,
    pub fuel_price: u32,
    pub payload_value: u32,
}

pub struct ParticipantState {
    pub name: String,
    pub balance: u32,
}

#[derive(Default, Encode, Decode, TypeInfo)]
pub struct LaunchSiteState {
    pub name: String,
    pub current_session: Option<CurrentSesionInfo>,
    pub participants: Vec<(ActorId, String, u32)>,
}
