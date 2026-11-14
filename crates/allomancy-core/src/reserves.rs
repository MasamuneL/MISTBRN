use crate::{BurnState, Metal, PlayerState};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReserveRules {
    pub consumption_milli_per_tick: [u64; 8],
    pub base_capacity_milli: [u64; 8],
    pub reduction_milli: u16,
    pub capacity_bonus_milli: u16,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReserveAccounting {
    pub remainder: [u16; 8],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReserveError {
    ReductionLimitExceeded,
    CapacityBonusLimitExceeded,
    ZeroConsumption(Metal),
    ZeroCapacity(Metal),
    ArithmeticOverflow(Metal),
    ReserveAboveCapacity(Metal),
    LockedBurn(Metal),
    EmptyBurn(Metal),
    InvalidRemainder(Metal),
    NegativeCredit,
    CapacityExceeded(Metal),
}

impl ReserveRules {
    pub fn validate(&self) -> Result<(), ReserveError> {
        if self.reduction_milli > 200 {
            return Err(ReserveError::ReductionLimitExceeded);
        }
        if self.capacity_bonus_milli > 500 {
            return Err(ReserveError::CapacityBonusLimitExceeded);
        }
        for metal in Metal::ALL {
            if self.consumption_milli_per_tick[metal.index()] == 0 {
                return Err(ReserveError::ZeroConsumption(metal));
            }
            if self.base_capacity_milli[metal.index()] == 0 {
                return Err(ReserveError::ZeroCapacity(metal));
            }
            self.checked_capacity(metal)?;
        }
        Ok(())
    }

    pub fn capacity(&self, metal: Metal) -> Result<u64, ReserveError> {
        self.validate()?;
        self.checked_capacity(metal)
    }

    fn checked_capacity(&self, metal: Metal) -> Result<u64, ReserveError> {
        let capacity = u128::from(self.base_capacity_milli[metal.index()])
            * (1_000 + u128::from(self.capacity_bonus_milli))
            / 1_000;
        u64::try_from(capacity).map_err(|_| ReserveError::ArithmeticOverflow(metal))
    }
}

pub fn validate_reserves(
    state: &PlayerState,
    accounting: &ReserveAccounting,
    rules: &ReserveRules,
) -> Result<(), ReserveError> {
    rules.validate()?;
    for metal in Metal::ALL {
        let i = metal.index();
        let value = state.metals[i];
        if value.reserve_milli > rules.checked_capacity(metal)? {
            return Err(ReserveError::ReserveAboveCapacity(metal));
        }
        if value.burn != BurnState::Off {
            if !value.unlocked {
                return Err(ReserveError::LockedBurn(metal));
            }
            if value.reserve_milli == 0 {
                return Err(ReserveError::EmptyBurn(metal));
            }
        }
        if accounting.remainder[i] >= 1_000
            || (value.reserve_milli == 0 && accounting.remainder[i] != 0)
        {
            return Err(ReserveError::InvalidRemainder(metal));
        }
    }
    Ok(())
}

/// Internal trusted helper; the caller verifies authority and transaction identity.
pub fn credit_reserve(
    state: &mut PlayerState,
    accounting: &ReserveAccounting,
    rules: &ReserveRules,
    metal: Metal,
    amount_milli: i64,
) -> Result<(), ReserveError> {
    validate_reserves(state, accounting, rules)?;
    let amount = u64::try_from(amount_milli).map_err(|_| ReserveError::NegativeCredit)?;
    let next = state.metals[metal.index()]
        .reserve_milli
        .checked_add(amount)
        .ok_or(ReserveError::ArithmeticOverflow(metal))?;
    if next > rules.checked_capacity(metal)? {
        return Err(ReserveError::CapacityExceeded(metal));
    }
    state.metals[metal.index()].reserve_milli = next;
    Ok(())
}

/// Consumes trusted elapsed ticks without changing `last_tick` or authenticating callers.
pub fn consume_reserves(
    state: &mut PlayerState,
    accounting: &mut ReserveAccounting,
    rules: &ReserveRules,
    elapsed_ticks: u64,
) -> Result<Vec<Metal>, ReserveError> {
    validate_reserves(state, accounting, rules)?;
    let mut next_metals = state.metals;
    let mut next_remainder = accounting.remainder;
    let mut exhausted = Vec::new();
    for metal in Metal::ALL {
        let i = metal.index();
        let multiplier = match state.metals[i].burn {
            BurnState::Off => continue,
            BurnState::Burning => 1,
            BurnState::Flared => 3,
        };
        let numerator = u128::from(rules.consumption_milli_per_tick[i])
            .checked_mul(1_000 - u128::from(rules.reduction_milli))
            .and_then(|n| n.checked_mul(multiplier))
            .and_then(|n| n.checked_mul(u128::from(elapsed_ticks)))
            .and_then(|n| n.checked_add(u128::from(accounting.remainder[i])))
            .ok_or(ReserveError::ArithmeticOverflow(metal))?;
        let consumed = u64::try_from(numerator / 1_000)
            .map_err(|_| ReserveError::ArithmeticOverflow(metal))?;
        if consumed >= state.metals[i].reserve_milli {
            next_metals[i].reserve_milli = 0;
            next_metals[i].burn = BurnState::Off;
            next_remainder[i] = 0;
            exhausted.push(metal);
        } else {
            next_metals[i].reserve_milli -= consumed;
            next_remainder[i] = (numerator % 1_000) as u16;
        }
    }
    state.metals = next_metals;
    accounting.remainder = next_remainder;
    Ok(exhausted)
}

pub fn clear_reserves_on_death(state: &mut PlayerState, accounting: &mut ReserveAccounting) {
    for metal in Metal::ALL {
        state.metals[metal.index()].reserve_milli = 0;
        state.metals[metal.index()].burn = BurnState::Off;
    }
    accounting.remainder = [0; 8];
}
