//! The core and the team of agents (ADR-034): the tools of the institution an agent may ask for, the agents of the
//! core (the Assistant and the Capturist) and the proposals the AI leaves for a person to accept. The loop itself
//! lives in the base (`ai::agent`); the modules never know the AI exists, so their tools are registered here, through
//! their `api`.

pub mod assistant;
pub mod capturist;
pub mod proposals;
pub mod tools;
