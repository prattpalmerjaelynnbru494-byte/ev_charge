#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, Symbol};

/// Storage keys used by the EV charging contract.
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Station(Symbol),
    Session(Symbol),
    OperatorRevenue(Address),
}

/// Status codes returned for a charging session.
#[derive(Clone, PartialEq)]
#[contracttype]
pub enum SessionStatus {
    Active = 0,
    AwaitingPayment = 1,
    Paid = 2,
    Disputed = 3,
    Cancelled = 4,
}

/// On-chain details for a registered EV charging station.
#[derive(Clone)]
#[contracttype]
pub struct Station {
    pub operator: Address,
    pub location: String,
    pub rate_per_kwh: u32, // price in micro-units per kWh (e.g. USDC stroops)
}

/// On-chain details for a single charging session.
#[derive(Clone)]
#[contracttype]
pub struct Session {
    pub driver: Address,
    pub station: Symbol,
    pub status: SessionStatus,
    pub kwh: u32,
    pub amount_due: u32,
    pub dispute_reason: String,
}

#[contract]
pub struct EvCharge;

#[contractimpl]
impl EvCharge {
    /// Register a new EV charging station under an operator.
    /// The operator sets a per-kWh rate (in micro-units) and a location label.
    pub fn register_station(
        env: Env,
        operator: Address,
        station_id: Symbol,
        location: String,
        rate_per_kwh: u32,
    ) {
        operator.require_auth();

        if env.storage().instance().has(&DataKey::Station(station_id.clone())) {
            panic!("Station already registered");
        }
        if rate_per_kwh == 0 {
            panic!("Rate must be greater than zero");
        }

        let station = Station {
            operator: operator.clone(),
            location,
            rate_per_kwh,
        };
        env.storage()
            .instance()
            .set(&DataKey::Station(station_id), &station);
    }

    /// Driver starts a new charging session at a known station.
    /// Stores the session in `Active` state with zero kWh delivered.
    pub fn start_session(env: Env, driver: Address, station_id: Symbol, session_id: Symbol) {
        driver.require_auth();

        if !env.storage().instance().has(&DataKey::Station(station_id.clone())) {
            panic!("Unknown station");
        }
        if env.storage().instance().has(&DataKey::Session(session_id.clone())) {
            panic!("Session already exists");
        }

        let session = Session {
            driver: driver.clone(),
            station: station_id,
            status: SessionStatus::Active,
            kwh: 0,
            amount_due: 0,
            dispute_reason: String::from_str(&env, ""),
        };
        env.storage()
            .instance()
            .set(&DataKey::Session(session_id), &session);
    }

    /// Station reports that the driver finished charging. Computes the amount
    /// due from kWh delivered and rate, then marks the session as awaiting payment.
    pub fn end_session(env: Env, station: Address, session_id: Symbol, kwh: u32) -> u32 {
        station.require_auth();

        let mut session: Session = env
            .storage()
            .instance()
            .get(&DataKey::Session(session_id.clone()))
            .expect("Session not found");

        if session.status != SessionStatus::Active {
            panic!("Session not active");
        }

        let station_data: Station = env
            .storage()
            .instance()
            .get(&DataKey::Station(session.station.clone()))
            .expect("Station missing");

        if station_data.operator != station {
            panic!("Only the station operator can end the session");
        }

        session.kwh = kwh;
        // amount = kwh * rate; safe multiplication using u64 intermediate.
        session.amount_due = (kwh as u64 * station_data.rate_per_kwh as u64) as u32;
        session.status = SessionStatus::AwaitingPayment;
        env.storage()
            .instance()
            .set(&DataKey::Session(session_id), &session);
        session.amount_due
    }

    /// Driver pays the amount due for a completed session. Credits the
    /// station operator's on-chain revenue record and marks the session as paid.
    pub fn pay(env: Env, driver: Address, session_id: Symbol) -> u32 {
        driver.require_auth();

        let mut session: Session = env
            .storage()
            .instance()
            .get(&DataKey::Session(session_id.clone()))
            .expect("Session not found");

        if session.driver != driver {
            panic!("Only the driver can pay");
        }
        if session.status != SessionStatus::AwaitingPayment {
            panic!("Session not awaiting payment");
        }

        session.status = SessionStatus::Paid;
        env.storage()
            .instance()
            .set(&DataKey::Session(session_id.clone()), &session);

        // Credit operator revenue.
        let station_data: Station = env
            .storage()
            .instance()
            .get(&DataKey::Station(session.station))
            .expect("Station missing");
        let key = DataKey::OperatorRevenue(station_data.operator.clone());
        let current: u64 = env.storage().instance().get(&key).unwrap_or(0u64);
        env.storage()
            .instance()
            .set(&key, &(current + session.amount_due as u64));

        session.amount_due
    }

    /// Driver disputes a session that is awaiting payment. The session is
    /// frozen in `Disputed` state with the reason recorded on-chain.
    pub fn dispute(env: Env, driver: Address, session_id: Symbol, reason: String) {
        driver.require_auth();

        let mut session: Session = env
            .storage()
            .instance()
            .get(&DataKey::Session(session_id.clone()))
            .expect("Session not found");

        if session.driver != driver {
            panic!("Only the driver can dispute");
        }
        if session.status != SessionStatus::AwaitingPayment {
            panic!("Only awaiting-payment sessions can be disputed");
        }

        session.status = SessionStatus::Disputed;
        session.dispute_reason = reason;
        env.storage()
            .instance()
            .set(&DataKey::Session(session_id), &session);
    }

    /// Return the numeric status code for a session (0=Active, 1=AwaitingPayment,
    /// 2=Paid, 3=Disputed, 4=Cancelled). Useful for off-chain UIs.
    pub fn get_session_status(env: Env, session_id: Symbol) -> u32 {
        let session: Session = env
            .storage()
            .instance()
            .get(&DataKey::Session(session_id))
            .expect("Session not found");
        match session.status {
            SessionStatus::Active => 0,
            SessionStatus::AwaitingPayment => 1,
            SessionStatus::Paid => 2,
            SessionStatus::Disputed => 3,
            SessionStatus::Cancelled => 4,
        }
    }

    /// Look up a registered station's operator and rate.
    pub fn get_station(env: Env, station_id: Symbol) -> Station {
        env.storage()
            .instance()
            .get(&DataKey::Station(station_id))
            .expect("Station not found")
    }

    /// Read the lifetime revenue (in micro-units) credited to an operator.
    pub fn get_operator_revenue(env: Env, operator: Address) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::OperatorRevenue(operator))
            .unwrap_or(0u64)
    }
}
