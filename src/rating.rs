//! Glicko-2 ratings, as used by Lichess.

use std::f64::consts::PI;

const SCALE: f64 = 173.7178;
const TAU: f64 = 0.5;
const EPSILON: f64 = 1e-6;

/// The result of a game from one player's point of view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameResult {
    Win,
    Loss,
    Draw,
}

impl GameResult {
    fn score(self) -> f64 {
        match self {
            GameResult::Win => 1.0,
            GameResult::Draw => 0.5,
            GameResult::Loss => 0.0,
        }
    }
}

/// A Glicko-2 rating: value, rating deviation and volatility.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rating {
    pub rating: f64,
    pub deviation: f64,
    pub volatility: f64,
}

impl Default for Rating {
    fn default() -> Self {
        Rating {
            rating: 1500.0,
            deviation: 350.0,
            volatility: 0.06,
        }
    }
}

impl Rating {
    pub fn new(rating: f64, deviation: f64, volatility: f64) -> Rating {
        Rating {
            rating,
            deviation,
            volatility,
        }
    }

    fn mu(&self) -> f64 {
        (self.rating - 1500.0) / SCALE
    }

    fn phi(&self) -> f64 {
        self.deviation / SCALE
    }

    /// Applies one rating period against a set of opponents.
    pub fn update(&self, results: &[(Rating, GameResult)]) -> Rating {
        if results.is_empty() {
            return *self;
        }

        let mu = self.mu();
        let phi = self.phi();

        let mut variance_inverse = 0.0;
        let mut delta_sum = 0.0;
        for (opponent, result) in results {
            let mu_j = opponent.mu();
            let phi_j = opponent.phi();
            let g = 1.0 / (1.0 + 3.0 * phi_j * phi_j / (PI * PI)).sqrt();
            let e = 1.0 / (1.0 + (-g * (mu - mu_j)).exp());
            let s = result.score();
            variance_inverse += g * g * e * (1.0 - e);
            delta_sum += g * (s - e);
        }

        let variance = 1.0 / variance_inverse;
        let delta = variance * delta_sum;
        let new_volatility = self.solve_volatility(delta, phi, variance);

        let phi_star = (phi * phi + new_volatility * new_volatility).sqrt();
        let new_phi = 1.0 / (1.0 / (phi_star * phi_star) + 1.0 / variance).sqrt();
        let new_mu = mu + new_phi * new_phi * delta_sum;

        Rating {
            rating: 1500.0 + SCALE * new_mu,
            deviation: SCALE * new_phi,
            volatility: new_volatility,
        }
    }

    fn solve_volatility(&self, delta: f64, phi: f64, variance: f64) -> f64 {
        let a = (phi * phi).ln();
        let f = |x: f64| {
            let e = x.exp();
            (e * (delta * delta - phi * phi - variance - e))
                / (2.0 * (phi * phi + variance + e).powi(2))
                - (x - a) / (TAU * TAU)
        };

        let mut lo = a;
        let mut hi = if delta * delta > phi * phi + variance {
            (delta * delta - phi * phi - variance).ln()
        } else {
            a - TAU
        };
        let mut f_lo = f(lo);
        let mut f_hi = f(hi);
        while f_lo * f_hi > 0.0 {
            hi -= TAU;
            f_hi = f(hi);
        }

        while (hi - lo).abs() > EPSILON {
            let c = lo + (lo - hi) * f_lo / (f_hi - f_lo);
            let f_c = f(c);
            if f_c * f_hi <= 0.0 {
                lo = hi;
                f_lo = f_hi;
            } else {
                f_lo /= 2.0;
            }
            hi = c;
            f_hi = f_c;
        }
        hi.exp()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_rating_is_1500() {
        assert_eq!(Rating::default().rating, 1500.0);
    }

    #[test]
    fn winning_raises_the_rating() {
        let player = Rating::new(1500.0, 200.0, 0.06);
        let opponent = Rating::new(1500.0, 30.0, 0.06);
        let updated = player.update(&[(opponent, GameResult::Win)]);
        assert!(updated.rating > player.rating);
        assert!(updated.deviation < player.deviation);
    }

    #[test]
    fn losing_lowers_the_rating() {
        let player = Rating::new(1500.0, 200.0, 0.06);
        let opponent = Rating::new(1500.0, 30.0, 0.06);
        let updated = player.update(&[(opponent, GameResult::Loss)]);
        assert!(updated.rating < player.rating);
    }

    #[test]
    fn an_empty_period_leaves_the_rating_unchanged() {
        let player = Rating::new(1400.0, 100.0, 0.06);
        assert_eq!(player.update(&[]), player);
    }
}
