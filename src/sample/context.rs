//! `get_next_context` — pure selection of the next slot to queue, factoring
//! random-bank, random-end modes etc. Returns a *copy* of the slot with
//! sampler-mode mutations applied; the actual filesystem path is unchanged.

use rand::Rng;

use crate::sample::modes::apply_rand_start;
use crate::state::{Bank, LoadNext, SamplerSettings, Slot};

/// Pick the next slot to queue. Currently chooses the slot directly after
/// `current_slot_index` (wrapping). Random modes are layered on top.
pub fn get_next_context<R: Rng>(
    bank: &Bank,
    current_slot_index: u8,
    settings: &SamplerSettings,
    rng: &mut R,
) -> Option<Slot> {
    let n = bank.slots.len();
    if n == 0 {
        return None;
    }

    if settings.load_next == LoadNext::Random {
        let occupied: Vec<&Slot> = bank.slots.iter().flatten().collect();
        if occupied.is_empty() {
            return None;
        }
        let pick = rng.gen_range(0..occupied.len());
        let mut s = occupied[pick].clone();
        apply_rand_start(&mut s, settings, rng);
        return Some(s);
    }

    // Wrap around looking for the next non-empty slot.
    for offset in 1..=n {
        let idx = (current_slot_index as usize + offset) % n;
        if let Some(s) = &bank.slots[idx] {
            let mut s = s.clone();
            apply_rand_start(&mut s, settings, rng);
            return Some(s);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::SourceKind;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn s(name: &str) -> Slot {
        Slot {
            source: SourceKind::File(format!("/clips/{}", name).into()),
            name: name.into(),
            start: 0.0,
            end: 1.0,
            length: 10.0,
            rate: 1.0,
        }
    }

    #[test]
    fn picks_next_non_empty_slot() {
        let mut b = Bank::empty();
        b.slots[0] = Some(s("a"));
        b.slots[3] = Some(s("d"));
        let mut rng = ChaCha8Rng::seed_from_u64(0);
        let got = get_next_context(&b, 0, &SamplerSettings::default(), &mut rng).unwrap();
        assert_eq!(got.name, "d");
    }

    #[test]
    fn wraps_to_earlier_slots() {
        let mut b = Bank::empty();
        b.slots[1] = Some(s("b"));
        let mut rng = ChaCha8Rng::seed_from_u64(0);
        let got = get_next_context(&b, 5, &SamplerSettings::default(), &mut rng).unwrap();
        assert_eq!(got.name, "b");
    }

    #[test]
    fn returns_none_when_bank_empty() {
        let b = Bank::empty();
        let mut rng = ChaCha8Rng::seed_from_u64(0);
        assert!(get_next_context(&b, 0, &SamplerSettings::default(), &mut rng).is_none());
    }

    #[test]
    fn load_next_random_picks_from_occupied_slots() {
        let mut b = Bank::empty();
        b.slots[0] = Some(s("a"));
        b.slots[2] = Some(s("c"));
        b.slots[4] = Some(s("e"));
        let mut settings = SamplerSettings::default();
        settings.load_next = LoadNext::Random;
        // Run many times; all results must be from the occupied set.
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        for _ in 0..50 {
            let got = get_next_context(&b, 0, &settings, &mut rng).unwrap();
            assert!(["a", "c", "e"].contains(&got.name.as_str()));
        }
    }
}
