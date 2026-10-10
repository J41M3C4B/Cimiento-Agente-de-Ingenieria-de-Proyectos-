//! The core and the team of agents (ADR-034): the tools of the institution an agent may ask for, and the proposals
//! the AI leaves for a person to accept. The loop itself lives in the base (`ai::agent`); the modules never know the
//! AI exists, so their tools are registered here, through their `api`.

// the assistant (IA3) and the capturist (IA4) are their first users in the app; until then only the tests use them
#[cfg_attr(not(test), allow(dead_code))]
pub mod proposals;
#[cfg_attr(not(test), allow(dead_code))]
pub mod tools;
