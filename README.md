# ev_charge

## Project Title
ev_charge — On-Chain EV Charging Session Billing

## Project Description
Electric vehicle drivers today depend on closed, operator-specific apps to pay for charging, which fragments receipts, hides pricing, and makes cross-network travel painful. `ev_charge` is a Stellar / Soroban smart contract that records the full lifecycle of a charging session — station registration, session start, kWh delivery, payment, and dispute — on a public ledger. Drivers get a tamper-proof receipt priced by the station's per-kWh rate, and operators accumulate verifiable revenue they can audit from any Stellar wallet.

## Project Vision
The long-term goal is a trust-minimized charging network where any compatible station anywhere in the world can be paid by any compatible vehicle wallet, with no proprietary app, no hidden fees, and no operator-controlled black box. `ev_charge` is the first building block: a shared, open billing primitive that roaming EV drivers, fleet operators, and energy providers can all build on top of.

## Key Features
- **Station Registry** — Operators register their station ID, human-readable location, and a per-kWh rate in micro-units.
- **Session Lifecycle** — Drivers open a session, the station closes it with the measured kWh, and the contract computes `amount_due = kWh * rate_per_kwh` automatically.
- **Per-Driver Payment** — The driver authorizes the payment; the operator's on-chain revenue counter is credited in the same transaction.
- **Dispute Path** — A driver who disagrees with the reported kWh can freeze the session in `Disputed` state and attach a reason string for an off-chain arbiter to review.
- **Public Read API** — Anyone can query session status, station metadata, and operator revenue without authentication.
- **Auth-Gated Writes** — Every state-changing call uses `require_auth()` so only the driver or operator can move their session forward.

## Contract

- **Network:** Stellar Testnet (Public)
- **Scope:** travel dApp — see `contracts/ev_charge/src/lib.rs` for the full ev_charge business logic.
- **Functions exposed:** see `Key Features` above and the `pub fn` list in `lib.rs`.
- **Contract ID:** `<CD6LMRE3FMQKKCZ2LOGXHYAURTFOWREXQAUEW7VGYKTR6B5D2YWLZ4CH>`
- **Explorer template:** `https://stellar.expert/explorer/testnet/tx/cddfb4e6bc800459c746c7d8954c97d805b5bd7c8e37c136481e15f7dd1411c1`
- **Screenshot of deployed contract on Stellar Expert:**
  `_(Screenshot of the contract page on Stellar Expert will appear here after deploy.)_`


## Future Scope
- **Real XLM / USDC Settlement** — Wire the `pay` function to a Stellar token contract so the actual transfer of value is enforced by the ledger, not just bookkeeping.
- **Oracle-Driven kWh Telemetry** — Accept kWh readings from a signed oracle account so the station's own operator cannot unilaterally inflate consumption.
- **On-Chain Dispute Resolution** — Add a multi-sig or simple DAO of arbiters that can release funds back to the driver or forward them to the operator based on the recorded reason.
- **Roaming & Interoperability** — Add a `roaming_partner` field so sessions started on one network can be settled by a partner operator.
- **Frontend dApp** — A Freighter-wallet-enabled web UI for drivers to scan a station QR, watch the session tick, and pay in one click.
- **Carbon Credit Hooks** — At `pay` time, mint or burn a carbon-credit token proportional to kWh delivered for green-energy accounting.

## Profile

- **Name:** <!-- Fill github name -->
- **Project:** `ev_charge` (travel)
- **Built with:** Soroban SDK 25, Rust, Stellar Testnet
