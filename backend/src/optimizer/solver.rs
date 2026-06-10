//! Resolution du probleme de decoupe 1D (cutting stock) avec :
//! - plusieurs longueurs de stock (dispo limitee ou illimitee),
//! - trait de scie (kerf) par coupe,
//! - seuil de chute reutilisable.
//!
//! Strategie : plusieurs heuristiques gloutonnes (FFD/BFD x choix de stock),
//! puis recherche locale (elimination de barre + reduction du stock) sous
//! budget de temps. On retient la meilleure solution.
//!
//! Objectif (du plus important au moins important) :
//!   1. placer le plus de pieces possible (moins de non-placees),
//!   2. minimiser la matiere brute totale consommee (= residuel le plus court),
//!   3. moins de barres,
//!   4. concentrer la perte (chute reutilisable maximale, perte reelle minimale).

use std::collections::HashMap;
use std::time::Instant;

use super::model::*;

#[derive(Clone)]
struct Instance {
    part_id: String,
    label: String,
    length_um: i64,
}

impl Instance {
    fn to_cut(&self) -> PlacedCut {
        PlacedCut {
            part_id: self.part_id.clone(),
            label: self.label.clone(),
            length_um: self.length_um,
        }
    }
}

#[derive(Clone, Copy)]
enum Fit {
    Best,
    First,
}

#[derive(Clone, Copy)]
enum StockPick {
    SmallestFit,
    Largest,
}

/// Point d'entree principal.
pub fn solve(problem: &Problem) -> Solution {
    let kerf = problem.settings.kerf_um.max(0);
    let threshold = problem.settings.reusable_threshold_um.max(0);

    let mut instances: Vec<Instance> = Vec::new();
    for p in &problem.parts {
        for _ in 0..p.qty {
            instances.push(Instance {
                part_id: p.id.clone(),
                label: p.label.clone(),
                length_um: p.length_um,
            });
        }
    }
    instances.sort_by(|a, b| {
        b.length_um
            .cmp(&a.length_um)
            .then_with(|| a.part_id.cmp(&b.part_id))
    });

    let strategies = [
        (Fit::Best, StockPick::SmallestFit),
        (Fit::Best, StockPick::Largest),
        (Fit::First, StockPick::SmallestFit),
        (Fit::First, StockPick::Largest),
    ];

    let deadline = problem
        .settings
        .time_limit_ms
        .map(|ms| Instant::now() + std::time::Duration::from_millis(ms));

    let mut best: Option<Solution> = None;
    for (fit, pick) in strategies {
        let mut sol = greedy(problem, &instances, kerf, threshold, fit, pick);
        local_search(&mut sol, problem, kerf, threshold, deadline);
        if better(&sol, &best) {
            best = Some(sol);
        }
    }

    best.expect("au moins une strategie produit une solution")
}

/// Cle d'ordre total a minimiser lexicographiquement.
fn key(s: &Solution) -> (usize, i64, usize, i64) {
    (
        s.unplaced.len(),
        s.summary.total_stock_um,
        s.summary.total_bars,
        s.summary.real_waste_um,
    )
}

fn better(cand: &Solution, cur: &Option<Solution>) -> bool {
    match cur {
        None => true,
        Some(c) => key(cand) < key(c),
    }
}

fn greedy(
    problem: &Problem,
    instances: &[Instance],
    kerf: i64,
    threshold: i64,
    fit: Fit,
    pick: StockPick,
) -> Solution {
    let mut bars: Vec<BarPlan> = Vec::new();
    let mut avail = avail_map(&problem.stocks);
    let mut unplaced: Vec<PlacedCut> = Vec::new();

    for inst in instances {
        let target = match fit {
            Fit::Best => best_fit_bar(&bars, inst.length_um, kerf),
            Fit::First => first_fit_bar(&bars, inst.length_um, kerf),
        };
        if let Some(idx) = target {
            bars[idx].push(inst.to_cut(), kerf);
            continue;
        }
        if let Some(stock) = choose_stock(&problem.stocks, &avail, inst.length_um, kerf, pick) {
            let mut bar = BarPlan::new(stock);
            bar.push(inst.to_cut(), kerf);
            bars.push(bar);
            dec_avail(&mut avail, &stock.id);
        } else {
            unplaced.push(inst.to_cut());
        }
    }

    finish(bars, unplaced, threshold)
}

fn best_fit_bar(bars: &[BarPlan], len: i64, kerf: i64) -> Option<usize> {
    let mut best: Option<(usize, i64)> = None;
    for (i, b) in bars.iter().enumerate() {
        if b.can_fit(len, kerf) {
            let rem = b.remaining_um();
            match best {
                Some((_, br)) if br <= rem => {}
                _ => best = Some((i, rem)),
            }
        }
    }
    best.map(|(i, _)| i)
}

fn first_fit_bar(bars: &[BarPlan], len: i64, kerf: i64) -> Option<usize> {
    bars.iter().position(|b| b.can_fit(len, kerf))
}

fn choose_stock<'a>(
    stocks: &'a [StockType],
    avail: &HashMap<String, Option<u32>>,
    len: i64,
    kerf: i64,
    pick: StockPick,
) -> Option<&'a StockType> {
    let mut candidates: Vec<&StockType> = stocks
        .iter()
        .filter(|s| len + kerf <= s.length_um && has_avail(avail, &s.id))
        .collect();
    match pick {
        StockPick::SmallestFit => {
            candidates.sort_by(|a, b| a.length_um.cmp(&b.length_um).then(a.id.cmp(&b.id)))
        }
        StockPick::Largest => {
            candidates.sort_by(|a, b| b.length_um.cmp(&a.length_um).then(a.id.cmp(&b.id)))
        }
    }
    candidates.into_iter().next()
}

/// Recherche locale : alterne elimination de barre et reduction de stock
/// jusqu'a stabilite (ou expiration du budget de temps).
fn local_search(
    sol: &mut Solution,
    problem: &Problem,
    kerf: i64,
    threshold: i64,
    deadline: Option<Instant>,
) {
    loop {
        if let Some(d) = deadline {
            if Instant::now() >= d {
                break;
            }
        }
        let mut changed = eliminate_one(sol, kerf);
        changed |= shrink_stocks(sol, problem, kerf);
        if !changed {
            break;
        }
    }

    *sol = finish(
        std::mem::take(&mut sol.bars),
        std::mem::take(&mut sol.unplaced),
        threshold,
    );
}

/// Tente de vider la barre la moins remplie en redistribuant ses coupes
/// ailleurs. Renvoie `true` si une barre a ete supprimee.
fn eliminate_one(sol: &mut Solution, kerf: i64) -> bool {
    let mut order: Vec<usize> = (0..sol.bars.len()).collect();
    order.sort_by_key(|&i| sol.bars[i].used_um);
    for &t in &order {
        if try_eliminate(sol, t, kerf) {
            return true;
        }
    }
    false
}

fn try_eliminate(sol: &mut Solution, t: usize, kerf: i64) -> bool {
    let cuts = sol.bars[t].cuts.clone();
    let mut order: Vec<usize> = (0..cuts.len()).collect();
    order.sort_by(|&a, &b| cuts[b].length_um.cmp(&cuts[a].length_um));

    let n = sol.bars.len();
    let mut extra = vec![0i64; n]; // matiere (piece+kerf) reservee par barre
    let mut plan: Vec<(usize, usize)> = Vec::with_capacity(cuts.len());

    for &ci in &order {
        let len = cuts[ci].length_um;
        let mut chosen: Option<(usize, i64)> = None;
        for (bi, b) in sol.bars.iter().enumerate() {
            if bi == t {
                continue;
            }
            let rem = b.remaining_um() - extra[bi];
            if len + kerf <= rem {
                let after = rem - len - kerf;
                match chosen {
                    Some((_, best_after)) if best_after <= after => {}
                    _ => chosen = Some((bi, after)),
                }
            }
        }
        match chosen {
            Some((bi, _)) => {
                extra[bi] += len + kerf;
                plan.push((ci, bi));
            }
            None => return false,
        }
    }

    for (ci, bi) in plan {
        let cut = cuts[ci].clone();
        sol.bars[bi].push(cut, kerf);
    }
    sol.bars.remove(t);
    true
}

/// Reduit chaque barre au plus petit type de stock disponible qui contient
/// encore toutes ses coupes. Diminue la matiere brute consommee. Renvoie
/// `true` si au moins une barre a ete reduite.
fn shrink_stocks(sol: &mut Solution, problem: &Problem, _kerf: i64) -> bool {
    // Dispo restante = dispo totale - barres deja utilisees.
    let mut avail = avail_map(&problem.stocks);
    for b in &sol.bars {
        dec_avail(&mut avail, &b.stock_id);
    }

    // Traiter les plus grosses barres d'abord (libere les longs stocks).
    let mut order: Vec<usize> = (0..sol.bars.len()).collect();
    order.sort_by(|&a, &b| sol.bars[b].stock_length_um.cmp(&sol.bars[a].stock_length_um));

    let mut changed = false;
    for &i in &order {
        let consumed = sol.bars[i].consumed_um();
        let current_len = sol.bars[i].stock_length_um;
        let current_id = sol.bars[i].stock_id.clone();

        // Plus petit stock dispo qui contient `consumed` et plus court que l'actuel.
        let mut best: Option<&StockType> = None;
        for s in &problem.stocks {
            if s.length_um >= consumed
                && s.length_um < current_len
                && has_avail(&avail, &s.id)
            {
                match best {
                    Some(b) if b.length_um <= s.length_um => {}
                    _ => best = Some(s),
                }
            }
        }

        if let Some(s) = best {
            let s = s.clone();
            inc_avail(&mut avail, &current_id); // on rend l'ancien stock
            dec_avail(&mut avail, &s.id); // on consomme le nouveau
            sol.bars[i].set_stock(&s);
            changed = true;
        }
    }
    changed
}

// ---- utilitaires ----

fn avail_map(stocks: &[StockType]) -> HashMap<String, Option<u32>> {
    let mut m = HashMap::new();
    for s in stocks {
        m.entry(s.id.clone())
            .and_modify(|cur: &mut Option<u32>| {
                *cur = match (*cur, s.available) {
                    (Some(a), Some(b)) => Some(a + b),
                    _ => None,
                }
            })
            .or_insert(s.available);
    }
    m
}

fn has_avail(avail: &HashMap<String, Option<u32>>, id: &str) -> bool {
    match avail.get(id) {
        None => false,
        Some(None) => true,
        Some(Some(n)) => *n > 0,
    }
}

fn dec_avail(avail: &mut HashMap<String, Option<u32>>, id: &str) {
    if let Some(Some(n)) = avail.get_mut(id) {
        if *n > 0 {
            *n -= 1;
        }
    }
}

fn inc_avail(avail: &mut HashMap<String, Option<u32>>, id: &str) {
    if let Some(Some(n)) = avail.get_mut(id) {
        *n += 1;
    }
}

fn finish(mut bars: Vec<BarPlan>, unplaced: Vec<PlacedCut>, threshold: i64) -> Solution {
    for b in &mut bars {
        b.finalize(threshold);
    }
    bars.sort_by(|a, b| {
        a.stock_length_um
            .cmp(&b.stock_length_um)
            .then(b.used_um.cmp(&a.used_um))
    });

    let summary = build_summary(&bars);
    let complete = unplaced.is_empty();
    Solution {
        bars,
        summary,
        complete,
        unplaced,
    }
}

fn build_summary(bars: &[BarPlan]) -> Summary {
    let mut by: HashMap<String, StockUsage> = HashMap::new();
    let mut total_stock = 0i64;
    let mut total_parts = 0i64;
    let mut total_kerf = 0i64;
    let mut total_offcut = 0i64;
    let mut reusable_offcut = 0i64;
    let mut reusable_count = 0usize;

    for b in bars {
        total_stock += b.stock_length_um;
        total_parts += b.used_um;
        total_kerf += b.kerf_total_um;
        total_offcut += b.offcut_um;
        if b.reusable {
            reusable_offcut += b.offcut_um;
            reusable_count += 1;
        }
        let e = by.entry(b.stock_id.clone()).or_insert_with(|| StockUsage {
            stock_id: b.stock_id.clone(),
            label: b.stock_label.clone(),
            length_um: b.stock_length_um,
            count: 0,
        });
        e.count += 1;
    }

    let real_waste = total_offcut - reusable_offcut;
    let utilization = if total_stock > 0 {
        (total_parts as f64) / (total_stock as f64) * 100.0
    } else {
        0.0
    };

    let mut bars_by_stock: Vec<StockUsage> = by.into_values().collect();
    bars_by_stock.sort_by_key(|u| u.length_um);

    Summary {
        total_bars: bars.len(),
        bars_by_stock,
        total_stock_um: total_stock,
        total_parts_um: total_parts,
        total_kerf_um: total_kerf,
        total_offcut_um: total_offcut,
        reusable_offcut_um: reusable_offcut,
        real_waste_um: real_waste,
        utilization_pct: (utilization * 100.0).round() / 100.0,
        reusable_count,
    }
}
