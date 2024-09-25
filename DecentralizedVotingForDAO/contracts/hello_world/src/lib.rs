#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Symbol, String, symbol_short};

// Structure to track the status of a proposal.
#[contracttype]
#[derive(Clone)]
pub struct ProposalStatus {
    pub proposal_id: u64,      // unique id for the proposal
    pub title: String,         // title of the proposal
    pub description: String,   // description of the proposal
    pub votes_for: u64,        // number of votes in favor
    pub votes_against: u64,    // number of votes against
    pub is_active: bool        // status if the proposal is still active
}

// Structure for tracking individual votes.
#[contracttype]
#[derive(Clone)]
pub struct Vote {
    pub proposal_id: u64,    // proposal id being voted on
    pub voter_id: u64,       // unique id for the voter
    pub in_favor: bool,      // whether the vote is in favor (true) or against (false)
    pub reward: u64          // reward given for voting
}

// Mapping for storing all proposals.
#[contracttype]
pub enum ProposalBook {
    Proposal(u64)  // Mapping proposal_id to ProposalStatus
}

// Mapping for storing all votes.
#[contracttype]
pub enum VoteBook {
    Vote(u64, u64)  // Mapping (proposal_id, voter_id) to Vote
}

const PROPOSAL_COUNT: Symbol = symbol_short!("PROPL_CO");
const REWARD_AMOUNT: u64 = 100; // Reward in Lumens (XLM) for voting

#[contract]
pub struct DAOContract;

#[contractimpl]
impl DAOContract {

    // Function to create a new proposal.
    pub fn create_proposal(env: Env, title: String, description: String) -> u64 {
        let mut proposal_count: u64 = env.storage().instance().get(&PROPOSAL_COUNT).unwrap_or(0);
        proposal_count += 1;

        let new_proposal = ProposalStatus {
            proposal_id: proposal_count,
            title: title.clone(),
            description: description.clone(),
            votes_for: 0,
            votes_against: 0,
            is_active: true
        };

        // Store the new proposal.
        env.storage().instance().set(&ProposalBook::Proposal(proposal_count), &new_proposal);
        env.storage().instance().set(&PROPOSAL_COUNT, &proposal_count);

        log!(&env, "Proposal Created: ID: {}, Title: {}", proposal_count, title);
        return proposal_count;
    }

    // Function to vote on a proposal.
    pub fn vote_on_proposal(env: Env, proposal_id: u64, voter_id: u64, in_favor: bool) {
        // Retrieve the proposal.
        let mut proposal = Self::view_proposal(env.clone(), proposal_id.clone());

        // Ensure the proposal is still active.
        if !proposal.is_active {
            log!(&env, "Proposal ID: {} is no longer active!", proposal_id);
            panic!("Proposal is closed!");
        }

        // Check if the voter has already voted.
        let existing_vote = Self::view_vote(env.clone(), proposal_id.clone(), voter_id.clone());
        if existing_vote.reward > 0 {
            log!(&env, "Voter ID: {} has already voted on Proposal ID: {}", voter_id, proposal_id);
            panic!("Already voted!");
        }

        // Register the vote and update the proposal status.
        if in_favor {
            proposal.votes_for += 1;
        } else {
            proposal.votes_against += 1;
        }

        // Record the vote.
        let new_vote = Vote {
            proposal_id,
            voter_id,
            in_favor,
            reward: REWARD_AMOUNT
        };
        env.storage().instance().set(&VoteBook::Vote(proposal_id.clone(), voter_id.clone()), &new_vote);

        // Update the proposal with the new vote count.
        env.storage().instance().set(&ProposalBook::Proposal(proposal_id.clone()), &proposal);

        log!(&env, "Voter ID: {} voted on Proposal ID: {}", voter_id, proposal_id);
    }

    // Function to close a proposal and finalize the result.
    pub fn close_proposal(env: Env, proposal_id: u64) {
        let mut proposal = Self::view_proposal(env.clone(), proposal_id.clone());

        // Ensure the proposal is still active.
        if !proposal.is_active {
            log!(&env, "Proposal ID: {} is already closed!", proposal_id);
            panic!("Proposal is closed!");
        }

        proposal.is_active = false;
        env.storage().instance().set(&ProposalBook::Proposal(proposal_id.clone()), &proposal);

        log!(&env, "Proposal ID: {} is now closed!", proposal_id);
    }

    // View the status of a specific proposal.
    pub fn view_proposal(env: Env, proposal_id: u64) -> ProposalStatus {
        let key = ProposalBook::Proposal(proposal_id.clone());
        env.storage().instance().get(&key).unwrap_or(ProposalStatus {
            proposal_id: 0,
            title: String::from_str(&env, "Not Found"),
            description: String::from_str(&env, "Not Found"),
            votes_for: 0,
            votes_against: 0,
            is_active: false
        })
    }

    // View the vote of a specific voter on a specific proposal.
    pub fn view_vote(env: Env, proposal_id: u64, voter_id: u64) -> Vote {
        let key = VoteBook::Vote(proposal_id.clone(), voter_id.clone());
        env.storage().instance().get(&key).unwrap_or(Vote {
            proposal_id: 0,
            voter_id: 0,
            in_favor: false,
            reward: 0
        })
    }
}
