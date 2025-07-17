//! API clients for various AI services.

use std::fmt;

pub mod openai;

/// Records the cost of an AI model.
///
/// Prices are given in cents as US currency per 1 million tokens.
#[derive(Debug)]
pub struct ModelCost {
    /// Cost per input
    input: f64,

    /// Cost per cached input
    cached_input: Option<f64>,

    /// Cost per output
    output: f64,
}

impl ModelCost {
    /// Records a new model cost.
    pub fn new(input: f64, cached_input: Option<f64>, output: f64) -> Self {
        Self {
            input,
            cached_input,
            output,
        }
    }

    /// The cost, in US cents, per 1 million input tokens.
    pub fn input(&self) -> Currency {
        Currency::USCents(self.input)
    }

    /// The cost, in US cents, per 1 million cached input tokens.
    pub fn cached_input(&self) -> Option<Currency> {
        match self.cached_input {
            Some(amount) => Some(Currency::USCents(amount)),
            None => None,
        }
    }

    /// The cost, in US cents, per 1 million output tokens.
    pub fn output(&self) -> Currency {
        Currency::USCents(self.output)
    }
}

/// Indicates an AI model or other data structure that has a cost.
pub trait HasCost {
    /// The cost of using the AI model.
    fn cost(&self) -> ModelCost;
}

/// Currencies.
#[derive(Debug)]
pub enum Currency {
    /// US cents.
    USCents(f64),
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Currency::USCents(amount) => f.write_fmt(format_args!("{amount}¢")),
        }
    }
}

#[cfg(test)]
mod tests {
    mod currency {
        use super::super::*;

        #[test]
        fn it_can_be_converted_to_a_string() {
            let c = Currency::USCents(150.0);
            assert_eq!(c.to_string(), "150¢");
        }
    }
}
