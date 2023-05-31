/* results

    baseline
    part 1 result: 994 in 145.5780515s
    memo refs: 834619249
    memo hits: 398856779 (48%)
    states visited: 834619249

    dp earlier hit
    part 1 result: 994 in 73.725909042s
    memo refs: 570205591
    memo hits: 272534745 (48%)
    states visited: 570205591

    u8
    part 1 result: 994 in 61.4443895s
    memo refs: 570205591
    memo hits: 272534745 (48%)
    states visited: 570205591

    u8 as u64
    part 1 result: 994 in 50.229123s
    memo refs: 570205591
    memo hits: 272534745 (48%)
    states visited: 570205591

    thonk geodes first
    part 1 result: 994 in 27.130415375s
    memo refs: 388886719
    memo hits: 187664247 (48%)
    states visited: 388886719

    thonk max_result
    part 1 result: 994 in 4.466372916s
    memo refs: 82181447
    memo hits: 37532477 (46%)
    states visited: 82181447

    thonk enough bots
    part 1 result: 994 in 544.071625ms
    memo refs: 12631450
    memo hits: 4919665 (39%)
    states visited: 12631450

    thonk don't idle
    part 1 result: 994 in 29.940291ms
    memo refs: 566020
    memo hits: 25144 (4%)
    states visited: 566020

    thonk no memo
    part 1 result: 994 in 2.720334ms
    memo refs: 0
    memo hits: 0 (NaN%)
    states visited: 674356


    baseline
    part 1 result: 994 in 139.228871125s
    dp earlier hit
    part 1 result: 994 in 72.762682833s
    u8
    part 1 result: 994 in 60.486425875s
    u8 as u64
    part 1 result: 994 in 45.978252375s
    thonk geodes first
    part 1 result: 994 in 26.093973166s
    thonk max_result
    part 1 result: 994 in 4.468414709s
    thonk enough bots
    part 1 result: 994 in 533.643333ms
    thonk don't idle
    part 1 result: 994 in 29.961834ms
    thonk no memo
    part 1 result: 994 in 2.635ms

    baseline
    part 1 result: 994 in 133.571285333s
    dp earlier hit
    part 1 result: 994 in 70.392901708s
    u8
    part 1 result: 994 in 58.661585875s
    u8 as u64
    part 1 result: 994 in 44.474583s
    thonk geodes first
    part 1 result: 994 in 25.377575s
    thonk max_result
    part 1 result: 994 in 4.358779459s
    thonk enough bots
    part 1 result: 994 in 528.285125ms
    thonk don't idle
    part 1 result: 994 in 29.662875ms
    thonk no memo
    part 1 result: 994 in 2.617959ms

    101,468,602,368
*/


use std::cell::UnsafeCell;


struct Stats {
    memo_refs: u64,
    memo_hits: u64,
    states_visited: u64,
    states_skipped: u128,
}

impl Stats {
    #[inline]
    const fn new() -> Self {
        Stats { memo_refs: 0, memo_hits: 0, states_visited: 0, states_skipped: 0 }
    }

    fn reset(&mut self) {
        *self = Self::new();
    }

    fn print(&self) {
        println!("memo refs: {}", self.memo_refs);
        println!("memo hits: {} ({:.0}%)", self.memo_hits, self.memo_hits as f64 / self.memo_refs as f64 * 100.0);
        println!("states visited: {}", self.states_visited);
        println!("states skipped: {}", self.states_skipped);
        println!();
    }
}

struct StatsNonsense {
    inner: UnsafeCell<Stats>,
}

impl StatsNonsense {
    fn with<F: FnOnce(&mut Stats)>(&self, f: F) {
        if DO_STATS {
            f(unsafe { &mut *self.inner.get() })
        }
    }
}

unsafe impl Sync for StatsNonsense {}


const DO_STATS: bool = 0==1;
static STATS: StatsNonsense = StatsNonsense { inner: UnsafeCell::new(Stats::new()) };


mod baseline {
    use regex::Regex;

    use super::STATS;

    #[derive(Clone, Copy, Debug)]
    pub struct Blueprint {
        pub id: u32,
        pub ore_robot: u32,
        pub clay_robot: u32,
        pub obsidian_robot: (u32, u32),
        pub geode_robot: (u32, u32),
    }

    pub fn parse(input: &str) -> Vec<Blueprint> {
        let mut result = Vec::with_capacity(128);

        let re = Regex::new(r"\d+").unwrap();
        for line in input.lines() {
            let mut numbers = re.find_iter(line);
            let mut next = || -> u32 {
                let number = numbers.next().unwrap();
                number.as_str().parse().unwrap()
            };

            let id = next();
            let ore_robot = next();
            let clay_robot = next();
            let obsidian_robot = (next(), next());
            let geode_robot = (next(), next());
            result.push(Blueprint {
                id,
                ore_robot,
                clay_robot,
                obsidian_robot,
                geode_robot,
            });
            assert!(numbers.next().is_none());
        }

        result
    }


    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    struct State {
        minute: u8,
        ore_robot:      u32,
        clay_robot:     u32,
        obsidian_robot: u32,
        geode_robot:    u32,
        ore:      u32,
        clay:     u32,
        obsidian: u32,
        geode:    u32,
    }

    impl State {
        fn new() -> Self {
            State {
                minute: 0,
                ore_robot:      1,
                clay_robot:     0,
                obsidian_robot: 0,
                geode_robot:    0,
                ore:      0,
                clay:     0,
                obsidian: 0,
                geode:    0,
            }
        }

        #[inline]
        fn can_build_ore_robot(&self, bp: &Blueprint) -> bool {
            self.ore >= bp.ore_robot
        }

        #[inline]
        fn can_build_clay_robot(&self, bp: &Blueprint) -> bool {
            self.ore >= bp.clay_robot
        }

        #[inline]
        fn can_build_obsidian_robot(&self, bp: &Blueprint) -> bool {
               self.ore  >= bp.obsidian_robot.0
            && self.clay >= bp.obsidian_robot.1
        }

        #[inline]
        fn can_build_geode_robot(&self, bp: &Blueprint) -> bool {
               self.ore      >= bp.geode_robot.0
            && self.obsidian >= bp.geode_robot.1
        }

        #[inline]
        fn build_ore_robot(self, bp: &Blueprint) -> Self {
            let mut result = self;
            result.ore -= bp.ore_robot;
            result.ore_robot += 1;
            return result;
        }

        #[inline]
        fn build_clay_robot(self, bp: &Blueprint) -> Self {
            let mut result = self;
            result.ore -= bp.clay_robot;
            result.clay_robot += 1;
            return result;
        }

        #[inline]
        fn build_obsidian_robot(self, bp: &Blueprint) -> Self {
            let mut result = self;
            result.ore  -= bp.obsidian_robot.0;
            result.clay -= bp.obsidian_robot.1;
            result.obsidian_robot += 1;
            return result;
        }

        #[inline]
        fn build_geode_robot(self, bp: &Blueprint) -> Self {
            let mut result = self;
            result.ore      -= bp.geode_robot.0;
            result.obsidian -= bp.geode_robot.1;
            result.geode_robot += 1;
            return result;
        }

        #[inline]
        fn step(self) -> Self {
            let mut this = self;
            this.minute += 1;
            this.ore      += this.ore_robot;
            this.clay     += this.clay_robot;
            this.obsidian += this.obsidian_robot;
            this.geode    += this.geode_robot;
            return this;
        }
    }


    pub mod v1 {
        use super::{Blueprint, State};

        pub fn solve(bp: &Blueprint, limit: u8) -> u32 {
            let mut state = State::new();
            for _ in 0..limit {
                if state.can_build_geode_robot(bp) {
                    state = state.step().build_geode_robot(bp);
                }
                else if state.can_build_obsidian_robot(bp) {
                    state = state.step().build_obsidian_robot(bp);
                }
                else if state.can_build_clay_robot(bp) {
                    state = state.step().build_clay_robot(bp);
                }
                else if state.can_build_ore_robot(bp) {
                    state = state.step().build_ore_robot(bp);
                }
                else {
                    state = state.step();
                }
            }

            state.geode
        }
    }

    pub mod v2 {
        use super::{STATS, Blueprint, State};

        fn solution(state: State, bp: &Blueprint, limit: u8) -> u32 {
            STATS.with(|s| { s.states_visited += 1 });
            STATS.with(|s| { 
                if s.states_visited % (128*1024*1024) == 0 {
                    println!("{}", s.states_visited);
                }
            });

            if state.minute == limit {
                return state.geode;
            }

            let mut result = 0;

            if state.can_build_geode_robot(bp) {
                result = result.max(solution(state.step().build_geode_robot(bp), bp, limit));
            }

            if state.can_build_obsidian_robot(bp) {
                result = result.max(solution(state.step().build_obsidian_robot(bp), bp, limit));
            }

            if state.can_build_clay_robot(bp) {
                result = result.max(solution(state.step().build_clay_robot(bp), bp, limit));
            }

            if state.can_build_ore_robot(bp) {
                result = result.max(solution(state.step().build_ore_robot(bp), bp, limit));
            }

            result = result.max(solution(state.step(), bp, limit));

            return result;
        }

        pub fn solve(bp: &Blueprint, limit: u8) -> u32 {
            solution(State::new(), bp, limit)
        }
    }

    pub mod v3 {
        use std::collections::HashMap;

        use super::{STATS, Blueprint, State};

        fn solution(state: State, bp: &Blueprint, limit: u8, memo: &mut HashMap<State, u32>) -> u32 {
            STATS.with(|s| {
                s.memo_refs += 1;
                s.states_visited += 1;
            });

            if let Some(result) = memo.get(&state).copied() {
                STATS.with(|s| { s.memo_hits += 1 });
                return result;
            }

            if state.minute == limit {
                let result = state.geode;
                memo.insert(state, result);
                return result;
            }

            let mut result = 0;

            if state.can_build_geode_robot(bp) {
                result = result.max(solution(state.step().build_geode_robot(bp), bp, limit, memo));
            }

            if state.can_build_obsidian_robot(bp) {
                result = result.max(solution(state.step().build_obsidian_robot(bp), bp, limit, memo));
            }

            if state.can_build_clay_robot(bp) {
                result = result.max(solution(state.step().build_clay_robot(bp), bp, limit, memo));
            }

            if state.can_build_ore_robot(bp) {
                result = result.max(solution(state.step().build_ore_robot(bp), bp, limit, memo));
            }

            result = result.max(solution(state.step(), bp, limit, memo));

            memo.insert(state, result);

            return result;
        }

        pub fn solve(bp: &Blueprint, limit: u8) -> u32 {
            let mut memo = HashMap::new();
            solution(State::new(), bp, limit, &mut memo)
        }
    }

    pub mod survivor {
        use std::collections::HashMap;

        use super::{STATS, Blueprint, State};

        fn solution(state: State, bp: &Blueprint, limit: u8, memo: &mut HashMap<State, (u32, u128)>) -> (u32, u128) {
            STATS.with(|s| {
                s.memo_refs += 1;
                s.states_visited += 1;
            });

            if let Some(result) = memo.get(&state).copied() {
                STATS.with(|s| {
                    s.memo_hits += 1;
                    s.states_skipped += result.1;
                });
                return result;
            }

            if state.minute == limit {
                let result = state.geode;
                memo.insert(state, (result, 1));
                return (result, 1);
            }

            let mut result = 0;
            let mut children = 0;

            if state.can_build_geode_robot(bp) {
                let (r, c) = solution(state.step().build_geode_robot(bp), bp, limit, memo);
                result = result.max(r);
                children += c;
            }

            if state.can_build_obsidian_robot(bp) {
                let (r, c) = solution(state.step().build_obsidian_robot(bp), bp, limit, memo);
                result = result.max(r);
                children += c;
            }

            if state.can_build_clay_robot(bp) {
                let (r, c) = solution(state.step().build_clay_robot(bp), bp, limit, memo);
                result = result.max(r);
                children += c;
            }

            if state.can_build_ore_robot(bp) {
                let (r, c) = solution(state.step().build_ore_robot(bp), bp, limit, memo);
                result = result.max(r);
                children += c;
            }

            {
                let (r, c) = solution(state.step(), bp, limit, memo);
                result = result.max(r);
                children += c;
            }

            memo.insert(state, (result, children));

            return (result, children);
        }

        pub fn solve(bp: &Blueprint, limit: u8) -> u32 {
            let mut memo = HashMap::new();
            let (r, c) = solution(State::new(), bp, limit, &mut memo);
            println!("children: {c}");
            return r;
        }
    }

    pub mod printer {
        use super::{STATS, Blueprint, State};

        fn solution(state: State, bp: &Blueprint, limit: u8) {
            println!("{:?}", state);

            if state.minute == limit {
                return;
            }

            #[inline]
            fn indent(state: State) {
                for _ in 0..state.minute + 1 {
                    print!("  ");
                }
            }

            if state.can_build_geode_robot(bp) {
                indent(state);
                print!("geode ");
                solution(state.step().build_geode_robot(bp), bp, limit);
            }

            if state.can_build_obsidian_robot(bp) {
                indent(state);
                print!("obsidian ");
                solution(state.step().build_obsidian_robot(bp), bp, limit);
            }

            if state.can_build_clay_robot(bp) {
                indent(state);
                print!("clay ");
                solution(state.step().build_clay_robot(bp), bp, limit);
            }

            if state.can_build_ore_robot(bp) {
                indent(state);
                print!("ore ");
                solution(state.step().build_ore_robot(bp), bp, limit);
            }

            indent(state);
            print!("wait ");
            solution(state.step(), bp, limit);
        }

        pub fn tree(bp: &Blueprint, limit: u8) {
            solution(State::new(), bp, limit);
        }
    }

    pub fn part_1<F: Fn(&Blueprint, u8) -> u32>(bps: &[Blueprint], f: F) {
        super::STATS.with(|s| s.reset());
        let t0 = std::time::Instant::now();
        let mut result = 0;
        for bp in bps {
            let geodes = f(bp, 24);
            // println!("geodes: {}", geodes);
            result += bp.id as u32 * geodes as u32;
        }
        println!("part 1 result: {} in {:?}", result, t0.elapsed());
        super::STATS.with(|s| s.print());
    }
}

mod pack {
    pub use super::baseline::{Blueprint, parse, part_1};

    use super::STATS;

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    struct Pack {
        ore_robot:      u32,
        clay_robot:     u32,
        obsidian_robot: u32,
        geode_robot:    u32,
        ore:      u32,
        clay:     u32,
        obsidian: u32,
        geode:    u32,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    struct State {
        minute: u8,
        pack:   Pack,
    }

    impl State {
        fn new() -> Self {
            State {
                minute: 0,
                pack: Pack {
                    ore_robot:      1,
                    clay_robot:     0,
                    obsidian_robot: 0,
                    geode_robot:    0,
                    ore:      0,
                    clay:     0,
                    obsidian: 0,
                    geode:    0,
                },
            }
        }

        #[inline]
        fn can_build_ore_robot(&self, bp: &Blueprint) -> bool {
            self.pack.ore >= bp.ore_robot
        }

        #[inline]
        fn can_build_clay_robot(&self, bp: &Blueprint) -> bool {
            self.pack.ore >= bp.clay_robot
        }

        #[inline]
        fn can_build_obsidian_robot(&self, bp: &Blueprint) -> bool {
               self.pack.ore  >= bp.obsidian_robot.0
            && self.pack.clay >= bp.obsidian_robot.1
        }

        #[inline]
        fn can_build_geode_robot(&self, bp: &Blueprint) -> bool {
               self.pack.ore      >= bp.geode_robot.0
            && self.pack.obsidian >= bp.geode_robot.1
        }

        #[inline]
        fn build_ore_robot(self, bp: &Blueprint) -> Self {
            let mut result = self;
            result.pack.ore -= bp.ore_robot;
            result.pack.ore_robot += 1;
            return result;
        }

        #[inline]
        fn build_clay_robot(self, bp: &Blueprint) -> Self {
            let mut result = self;
            result.pack.ore -= bp.clay_robot;
            result.pack.clay_robot += 1;
            return result;
        }

        #[inline]
        fn build_obsidian_robot(self, bp: &Blueprint) -> Self {
            let mut result = self;
            result.pack.ore  -= bp.obsidian_robot.0;
            result.pack.clay -= bp.obsidian_robot.1;
            result.pack.obsidian_robot += 1;
            return result;
        }

        #[inline]
        fn build_geode_robot(self, bp: &Blueprint) -> Self {
            let mut result = self;
            result.pack.ore      -= bp.geode_robot.0;
            result.pack.obsidian -= bp.geode_robot.1;
            result.pack.geode_robot += 1;
            return result;
        }

        #[inline]
        fn step(self) -> Self {
            let mut this = self;
            this.minute += 1;
            this.pack.ore      += this.pack.ore_robot;
            this.pack.clay     += this.pack.clay_robot;
            this.pack.obsidian += this.pack.obsidian_robot;
            this.pack.geode    += this.pack.geode_robot;
            return this;
        }
    }


    pub mod v1 {
        use std::collections::HashMap;

        use super::STATS;

        use super::{Blueprint, Pack, State};

        fn solution(state: State, bp: &Blueprint, limit: u8, memo: &mut HashMap<Pack, (u32, u8)>) -> u32 {
            STATS.with(|s| {
                s.memo_refs += 1;
                s.states_visited += 1;
            });

            if let Some((result, minute)) = memo.get(&state.pack).copied() {
                if state.minute >= minute {
                    STATS.with(|s| { s.memo_hits += 1 });

                    return result;
                }
            }

            #[inline]
            fn insert(memo: &mut HashMap<Pack, (u32, u8)>, state: &State, result: u32) {
                // note: for some reason, this is equivalent to simply overriding the result.
                //  ie: `memo.insert(pack, result)`
                //  could be, cause we're writing the results on the way up (going backwards in time).
                //  but not sure how that would work with "sibling branches".
                //  in any case, this isn't measurably slower, so who cares.
                memo.entry(state.pack)
                .and_modify(|(old_result, old_minute)| {
                    if state.minute < *old_minute {
                        *old_result = result;
                        *old_minute = state.minute;
                    }
                })
                .or_insert((result, state.minute));
            }

            if state.minute == limit {
                let result = state.pack.geode;
                insert(memo, &state, result);
                return result;
            }

            let mut result = 0;

            if state.can_build_geode_robot(bp) {
                result = result.max(solution(state.step().build_geode_robot(bp), bp, limit, memo));
            }

            if state.can_build_obsidian_robot(bp) {
                result = result.max(solution(state.step().build_obsidian_robot(bp), bp, limit, memo));
            }

            if state.can_build_clay_robot(bp) {
                result = result.max(solution(state.step().build_clay_robot(bp), bp, limit, memo));
            }

            if state.can_build_ore_robot(bp) {
                result = result.max(solution(state.step().build_ore_robot(bp), bp, limit, memo));
            }

            result = result.max(solution(state.step(), bp, limit, memo));

            insert(memo, &state, result);

            return result;
        }

        pub fn solve(bp: &Blueprint, limit: u8) -> u32 {
            let mut memo = HashMap::new();
            solution(State::new(), bp, limit, &mut memo)
        }

        pub fn solve_stats(bp: &Blueprint, limit: u8) -> u32 {
            let mut memo = HashMap::new();
            let result = solution(State::new(), bp, limit, &mut memo);

            println!("visited {} states", memo.len());
            let mut max_ore      = 0;
            let mut max_clay     = 0;
            let mut max_obsidian = 0;
            let mut max_geode    = 0;
            for (pack, _) in memo.iter() {
                max_ore      = max_ore.max(pack.ore);
                max_clay     = max_clay.max(pack.clay);
                max_obsidian = max_obsidian.max(pack.obsidian);
                max_geode    = max_geode.max(pack.geode);
            }
            println!("max ore: {}", max_ore);
            println!("max clay: {}", max_clay);
            println!("max obsidian: {}", max_obsidian);
            println!("max geode: {}", max_geode);

            return result;
        }
    }
}

mod pack_u8 {
    use regex::Regex;

    use super::STATS;

    #[derive(Clone, Copy, Debug)]
    pub struct Blueprint {
        pub id: u8,
        pub ore_robot: u8,
        pub clay_robot: u8,
        pub obsidian_robot: (u8, u8),
        pub geode_robot: (u8, u8),
    }

    pub fn parse(input: &str) -> Vec<Blueprint> {
        let mut result = Vec::with_capacity(128);

        let re = Regex::new(r"\d+").unwrap();
        for line in input.lines() {
            let mut numbers = re.find_iter(line);
            let mut next = || -> u8 {
                let number = numbers.next().unwrap();
                number.as_str().parse().unwrap()
            };

            let id = next();
            let ore_robot = next();
            let clay_robot = next();
            let obsidian_robot = (next(), next());
            let geode_robot = (next(), next());
            result.push(Blueprint {
                id,
                ore_robot,
                clay_robot,
                obsidian_robot,
                geode_robot,
            });
            assert!(numbers.next().is_none());
        }

        result
    }


    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    struct Pack {
        pub ore_robot:      u8,
        pub clay_robot:     u8,
        pub obsidian_robot: u8,
        pub geode_robot:    u8,
        pub ore:      u8,
        pub clay:     u8,
        pub obsidian: u8,
        pub geode:    u8,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    struct State {
        pub minute: u8,
        pub pack:   Pack,
    }

    impl State {
        pub fn new() -> Self {
            State {
                minute: 0,
                pack: Pack {
                    ore_robot:      1,
                    clay_robot:     0,
                    obsidian_robot: 0,
                    geode_robot:    0,
                    ore:      0,
                    clay:     0,
                    obsidian: 0,
                    geode:    0,
                },
            }
        }

        #[inline]
        pub fn can_build_ore_robot(&self, bp: &Blueprint) -> bool {
            self.pack.ore >= bp.ore_robot
        }

        #[inline]
        pub fn can_build_clay_robot(&self, bp: &Blueprint) -> bool {
            self.pack.ore >= bp.clay_robot
        }

        #[inline]
        pub fn can_build_obsidian_robot(&self, bp: &Blueprint) -> bool {
               self.pack.ore  >= bp.obsidian_robot.0
            && self.pack.clay >= bp.obsidian_robot.1
        }

        #[inline]
        pub fn can_build_geode_robot(&self, bp: &Blueprint) -> bool {
               self.pack.ore      >= bp.geode_robot.0
            && self.pack.obsidian >= bp.geode_robot.1
        }

        #[inline]
        pub fn build_ore_robot(self, bp: &Blueprint) -> Self {
            let mut result = self;
            result.pack.ore -= bp.ore_robot;
            result.pack.ore_robot += 1;
            return result;
        }

        #[inline]
        pub fn build_clay_robot(self, bp: &Blueprint) -> Self {
            let mut result = self;
            result.pack.ore -= bp.clay_robot;
            result.pack.clay_robot += 1;
            return result;
        }

        #[inline]
        pub fn build_obsidian_robot(self, bp: &Blueprint) -> Self {
            let mut result = self;
            result.pack.ore  -= bp.obsidian_robot.0;
            result.pack.clay -= bp.obsidian_robot.1;
            result.pack.obsidian_robot += 1;
            return result;
        }

        #[inline]
        pub fn build_geode_robot(self, bp: &Blueprint) -> Self {
            let mut result = self;
            result.pack.ore      -= bp.geode_robot.0;
            result.pack.obsidian -= bp.geode_robot.1;
            result.pack.geode_robot += 1;
            return result;
        }

        #[inline]
        pub fn step(self) -> Self {
            let mut this = self;
            this.minute += 1;
            this.pack.ore      += this.pack.ore_robot;
            this.pack.clay     += this.pack.clay_robot;
            this.pack.obsidian += this.pack.obsidian_robot;
            this.pack.geode    += this.pack.geode_robot;
            return this;
        }
    }


    pub mod v1 {
        use std::collections::HashMap;

        use super::{STATS, Blueprint, Pack, State};

        fn solution(state: State, bp: &Blueprint, limit: u8, memo: &mut HashMap<Pack, (u8, u8)>) -> u8 {
            STATS.with(|s| {
                s.memo_refs += 1;
                s.states_visited += 1;
            });

            if let Some((result, minute)) = memo.get(&state.pack).copied() {
                if state.minute >= minute {
                    STATS.with(|s| { s.memo_hits += 1 });
                    return result;
                }
            }

            #[inline]
            fn insert(memo: &mut HashMap<Pack, (u8, u8)>, state: &State, result: u8) {
                // note: for some reason, this is equivalent to simply overriding the result.
                //  ie: `memo.insert(pack, result)`
                //  could be, cause we're writing the results on the way up (going backwards in time).
                //  but not sure how that would work with "sibling branches".
                //  in any case, this isn't measurably slower, so who cares.
                memo.entry(state.pack)
                .and_modify(|(old_result, old_minute)| {
                    if state.minute < *old_minute {
                        *old_result = result;
                        *old_minute = state.minute;
                    }
                })
                .or_insert((result, state.minute));
            }

            if state.minute == limit {
                let result = state.pack.geode;
                insert(memo, &state, result);
                return result;
            }

            let mut result = 0;

            if state.can_build_geode_robot(bp) {
                result = result.max(solution(state.step().build_geode_robot(bp), bp, limit, memo));
            }

            if state.can_build_obsidian_robot(bp) {
                result = result.max(solution(state.step().build_obsidian_robot(bp), bp, limit, memo));
            }

            if state.can_build_clay_robot(bp) {
                result = result.max(solution(state.step().build_clay_robot(bp), bp, limit, memo));
            }

            if state.can_build_ore_robot(bp) {
                result = result.max(solution(state.step().build_ore_robot(bp), bp, limit, memo));
            }

            result = result.max(solution(state.step(), bp, limit, memo));

            insert(memo, &state, result);

            return result;
        }

        pub fn solve(bp: &Blueprint, limit: u8) -> u8 {
            let mut memo = HashMap::new();
            solution(State::new(), bp, limit, &mut memo)
        }
    }

    pub mod v2 {
        use std::collections::HashMap;

        use super::{STATS, Blueprint, State};

        fn solution(state: State, bp: &Blueprint, limit: u8, memo: &mut HashMap<u64, (u8, u8)>) -> u8 {
            STATS.with(|s| {
                s.memo_refs += 1;
                s.states_visited += 1;
            });

            let pack_64 = unsafe { core::mem::transmute(state.pack) };

            if let Some((result, minute)) = memo.get(&pack_64).copied() {
                if state.minute >= minute {
                    STATS.with(|s| { s.memo_hits += 1 });
                    return result;
                }
            }

            #[inline]
            fn insert(memo: &mut HashMap<u64, (u8, u8)>, state: &State, result: u8) {
                // note: for some reason, this is equivalent to simply overriding the result.
                //  ie: `memo.insert(pack, result)`
                //  could be, cause we're writing the results on the way up (going backwards in time).
                //  but not sure how that would work with "sibling branches".
                //  in any case, this isn't measurably slower, so who cares.
                memo.entry(unsafe { core::mem::transmute(state.pack) })
                .and_modify(|(old_result, old_minute)| {
                    if state.minute < *old_minute {
                        *old_result = result;
                        *old_minute = state.minute;
                    }
                })
                .or_insert((result, state.minute));
            }

            if state.minute == limit {
                let result = state.pack.geode;
                insert(memo, &state, result);
                return result;
            }

            let mut result = 0;

            if state.can_build_geode_robot(bp) {
                result = result.max(solution(state.step().build_geode_robot(bp), bp, limit, memo));
            }

            if state.can_build_obsidian_robot(bp) {
                result = result.max(solution(state.step().build_obsidian_robot(bp), bp, limit, memo));
            }

            if state.can_build_clay_robot(bp) {
                result = result.max(solution(state.step().build_clay_robot(bp), bp, limit, memo));
            }

            if state.can_build_ore_robot(bp) {
                result = result.max(solution(state.step().build_ore_robot(bp), bp, limit, memo));
            }

            result = result.max(solution(state.step(), bp, limit, memo));

            insert(memo, &state, result);

            return result;
        }

        pub fn solve(bp: &Blueprint, limit: u8) -> u8 {
            let mut memo = HashMap::new();
            solution(State::new(), bp, limit, &mut memo)
        }
    }

    pub fn part_1<F: Fn(&Blueprint, u8) -> u8>(bps: &[Blueprint], f: F) {
        super::STATS.with(|s| s.reset());
        let t0 = std::time::Instant::now();
        let mut result = 0;
        for bp in bps {
            let geodes = f(bp, 24);
            // println!("geodes: {}", geodes);
            result += bp.id as u32 * geodes as u32;
        }
        println!("part 1 result: {} in {:?}", result, t0.elapsed());
        super::STATS.with(|s| s.print());
    }
}

mod thonk {
    use regex::Regex;

    use super::STATS;

    #[derive(Clone, Copy, Debug)]
    pub struct Blueprint {
        pub id: u8,
        pub ore_robot: u8,
        pub clay_robot: u8,
        pub obsidian_robot: (u8, u8),
        pub geode_robot: (u8, u8),
        pub max_ore_cost: u8,
    }

    impl Blueprint {
        #[inline]
        fn max_ore_cost(&self) -> u8 {
            self.max_ore_cost
        }

        #[inline]
        fn max_clay_cost(&self) -> u8 {
            self.obsidian_robot.1
        }

        #[inline]
        fn max_obsidian_cost(&self) -> u8 {
            self.geode_robot.1
        }
    }

    pub fn parse(input: &str) -> Vec<Blueprint> {
        let mut result = Vec::with_capacity(128);

        let re = Regex::new(r"\d+").unwrap();
        for line in input.lines() {
            let mut numbers = re.find_iter(line);
            let mut next = || -> u8 {
                let number = numbers.next().unwrap();
                number.as_str().parse().unwrap()
            };

            let id = next();
            let ore_robot = next();
            let clay_robot = next();
            let obsidian_robot = (next(), next());
            let geode_robot = (next(), next());
            result.push(Blueprint {
                id,
                ore_robot,
                clay_robot,
                obsidian_robot,
                geode_robot,
                max_ore_cost: ore_robot.max(clay_robot).max(obsidian_robot.0).max(geode_robot.0),
            });
            assert!(numbers.next().is_none());
        }

        result
    }


    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    struct Pack {
        pub ore_robot:      u8,
        pub clay_robot:     u8,
        pub obsidian_robot: u8,
        pub geode_robot:    u8,
        pub ore:      u8,
        pub clay:     u8,
        pub obsidian: u8,
        pub geode:    u8,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    struct State {
        pub minute: u8,
        pub pack:   Pack,
    }

    impl State {
        pub fn new() -> Self {
            State {
                minute: 0,
                pack: Pack {
                    ore_robot:      1,
                    clay_robot:     0,
                    obsidian_robot: 0,
                    geode_robot:    0,
                    ore:      0,
                    clay:     0,
                    obsidian: 0,
                    geode:    0,
                },
            }
        }

        #[inline]
        pub fn can_build_ore_robot(&self, bp: &Blueprint) -> bool {
            self.pack.ore >= bp.ore_robot
        }

        #[inline]
        pub fn can_build_clay_robot(&self, bp: &Blueprint) -> bool {
            self.pack.ore >= bp.clay_robot
        }

        #[inline]
        pub fn can_build_obsidian_robot(&self, bp: &Blueprint) -> bool {
               self.pack.ore  >= bp.obsidian_robot.0
            && self.pack.clay >= bp.obsidian_robot.1
        }

        #[inline]
        pub fn can_build_geode_robot(&self, bp: &Blueprint) -> bool {
               self.pack.ore      >= bp.geode_robot.0
            && self.pack.obsidian >= bp.geode_robot.1
        }

        #[inline]
        pub fn build_ore_robot(self, bp: &Blueprint) -> Self {
            let mut result = self;
            result.pack.ore -= bp.ore_robot;
            result.pack.ore_robot += 1;
            return result;
        }

        #[inline]
        pub fn build_clay_robot(self, bp: &Blueprint) -> Self {
            let mut result = self;
            result.pack.ore -= bp.clay_robot;
            result.pack.clay_robot += 1;
            return result;
        }

        #[inline]
        pub fn build_obsidian_robot(self, bp: &Blueprint) -> Self {
            let mut result = self;
            result.pack.ore  -= bp.obsidian_robot.0;
            result.pack.clay -= bp.obsidian_robot.1;
            result.pack.obsidian_robot += 1;
            return result;
        }

        #[inline]
        pub fn build_geode_robot(self, bp: &Blueprint) -> Self {
            let mut result = self;
            result.pack.ore      -= bp.geode_robot.0;
            result.pack.obsidian -= bp.geode_robot.1;
            result.pack.geode_robot += 1;
            return result;
        }

        #[inline]
        pub fn step(self) -> Self {
            let mut this = self;
            this.minute += 1;
            this.pack.ore      += this.pack.ore_robot;
            this.pack.clay     += this.pack.clay_robot;
            this.pack.obsidian += this.pack.obsidian_robot;
            this.pack.geode    += this.pack.geode_robot;
            return this;
        }
    }


    pub mod v1 {
        use std::collections::HashMap;

        use super::{STATS, Blueprint, State};

        fn solution(state: State, bp: &Blueprint, limit: u8, memo: &mut HashMap<u64, (u8, u8)>, max_result: &mut u8) -> u8 {
            STATS.with(|s| {
                s.memo_refs += 1;
                s.states_visited += 1;
            });

            let pack_64 = unsafe { core::mem::transmute(state.pack) };

            if let Some((result, minute)) = memo.get(&pack_64).copied() {
                if state.minute >= minute {
                    STATS.with(|s| { s.memo_hits += 1 });

                    *max_result = (*max_result).max(result);
                    return result;
                }
            }

            #[inline]
            fn insert(memo: &mut HashMap<u64, (u8, u8)>, state: &State, result: u8) {
                // note: for some reason, this is equivalent to simply overriding the result.
                //  ie: `memo.insert(pack, result)`
                //  could be, cause we're writing the results on the way up (going backwards in time).
                //  but not sure how that would work with "sibling branches".
                //  in any case, this isn't measurably slower, so who cares.
                memo.entry(unsafe { core::mem::transmute(state.pack) })
                .and_modify(|(old_result, old_minute)| {
                    if state.minute < *old_minute {
                        *old_result = result;
                        *old_minute = state.minute;
                    }
                })
                .or_insert((result, state.minute));
            }

            // done?
            if state.minute == limit {
                let result = state.pack.geode;
                insert(memo, &state, result);
                *max_result = (*max_result).max(result);
                return result;
            }

            // can we even beat max_result anymore?
            {
                // number of turns remaining.
                let remaining = (limit - state.minute) as u32;

                let max_yield =
                      // future yield of current geode bots.
                      remaining * state.pack.geode_robot as u32
                      // max future yield, if we build one geode bot
                      // on all future turns.
                    + remaining*(remaining-1)/2;

                if state.pack.geode as u32 + max_yield <= *max_result as u32 {
                    // doesn't matter what we insert,
                    // we already have a better result.
                    insert(memo, &state, 0);
                    return 0;
                }
            }

            let mut result = 0;

            if state.can_build_geode_robot(bp) {
                result = result.max(solution(state.step().build_geode_robot(bp), bp, limit, memo, max_result));
            }

            if state.can_build_obsidian_robot(bp) {
                result = result.max(solution(state.step().build_obsidian_robot(bp), bp, limit, memo, max_result));
            }

            if state.can_build_clay_robot(bp) {
                result = result.max(solution(state.step().build_clay_robot(bp), bp, limit, memo, max_result));
            }

            if state.can_build_ore_robot(bp) {
                result = result.max(solution(state.step().build_ore_robot(bp), bp, limit, memo, max_result));
            }

            result = result.max(solution(state.step(), bp, limit, memo, max_result));

            insert(memo, &state, result);

            return result;
        }

        pub fn solve(bp: &Blueprint, limit: u8) -> u8 {
            let mut memo = HashMap::new();
            let mut max_result = 0;
            solution(State::new(), bp, limit, &mut memo, &mut max_result)
        }
    }

    pub mod v2 {
        use std::collections::HashMap;

        use super::{STATS, Blueprint, State};

        fn solution(state: State, bp: &Blueprint, limit: u8, memo: &mut HashMap<u64, (u8, u8)>, max_result: &mut u8) -> u8 {
            STATS.with(|s| {
                s.memo_refs += 1;
                s.states_visited += 1;
            });

            let pack_64 = unsafe { core::mem::transmute(state.pack) };

            if let Some((result, minute)) = memo.get(&pack_64).copied() {
                if state.minute >= minute {
                    STATS.with(|s| { s.memo_hits += 1 });

                    *max_result = (*max_result).max(result);
                    return result;
                }
            }

            #[inline]
            fn insert(memo: &mut HashMap<u64, (u8, u8)>, state: &State, result: u8) {
                // note: for some reason, this is equivalent to simply overriding the result.
                //  ie: `memo.insert(pack, result)`
                //  could be, cause we're writing the results on the way up (going backwards in time).
                //  but not sure how that would work with "sibling branches".
                //  in any case, this isn't measurably slower, so who cares.
                memo.entry(unsafe { core::mem::transmute(state.pack) })
                .and_modify(|(old_result, old_minute)| {
                    if state.minute < *old_minute {
                        *old_result = result;
                        *old_minute = state.minute;
                    }
                })
                .or_insert((result, state.minute));
            }

            // done?
            if state.minute == limit {
                let result = state.pack.geode;
                insert(memo, &state, result);
                *max_result = (*max_result).max(result);
                return result;
            }

            // can we even beat max_result anymore?
            {
                // number of turns remaining.
                let remaining = (limit - state.minute) as u32;

                let max_yield =
                      // future yield of current geode bots.
                      remaining * state.pack.geode_robot as u32
                      // max future yield, if we build one geode bot
                      // on all future turns.
                    + remaining*(remaining-1)/2;

                if state.pack.geode as u32 + max_yield <= *max_result as u32 {
                    // doesn't matter what we insert,
                    // we already have a better result.
                    insert(memo, &state, 0);
                    return 0;
                }
            }

            let mut result = 0;

            if state.can_build_geode_robot(bp) {
                result = result.max(solution(state.step().build_geode_robot(bp), bp, limit, memo, max_result));
            }

            if state.can_build_obsidian_robot(bp) {
                // can only build one bot per turn.
                // don't need more bots, if we're producing enough,
                // so we can build the most expensive bot on each turn.
                if state.pack.obsidian_robot < bp.max_obsidian_cost() {
                    result = result.max(solution(state.step().build_obsidian_robot(bp), bp, limit, memo, max_result));
                }
            }

            if state.can_build_clay_robot(bp) {
                if state.pack.clay_robot < bp.max_clay_cost() {
                    result = result.max(solution(state.step().build_clay_robot(bp), bp, limit, memo, max_result));
                }
            }

            if state.can_build_ore_robot(bp) {
                if state.pack.ore_robot < bp.max_ore_cost() {
                    result = result.max(solution(state.step().build_ore_robot(bp), bp, limit, memo, max_result));
                }
            }

            result = result.max(solution(state.step(), bp, limit, memo, max_result));

            insert(memo, &state, result);

            return result;
        }

        pub fn solve(bp: &Blueprint, limit: u8) -> u8 {
            let mut memo = HashMap::new();
            let mut max_result = 0;
            solution(State::new(), bp, limit, &mut memo, &mut max_result)
        }
    }

    pub mod v3 {
        use std::collections::HashMap;

        use super::{STATS, Blueprint, State};

        fn solution(state: State, bp: &Blueprint, limit: u8, memo: &mut HashMap<u64, (u8, u8)>, max_result: &mut u8,
            can_ore: bool, can_clay: bool, can_obsidian: bool
        ) -> u8 {
            STATS.with(|s| {
                s.memo_refs += 1;
                s.states_visited += 1;
            });

            let pack_64 = unsafe { core::mem::transmute(state.pack) };

            if let Some((result, minute)) = memo.get(&pack_64).copied() {
                if state.minute >= minute {
                    STATS.with(|s| { s.memo_hits += 1 });

                    *max_result = (*max_result).max(result);
                    return result;
                }
            }

            #[inline]
            fn insert(memo: &mut HashMap<u64, (u8, u8)>, state: &State, result: u8) {
                // note: for some reason, this is equivalent to simply overriding the result.
                //  ie: `memo.insert(pack, result)`
                //  could be, cause we're writing the results on the way up (going backwards in time).
                //  but not sure how that would work with "sibling branches".
                //  in any case, this isn't measurably slower, so who cares.
                memo.entry(unsafe { core::mem::transmute(state.pack) })
                .and_modify(|(old_result, old_minute)| {
                    if state.minute < *old_minute {
                        *old_result = result;
                        *old_minute = state.minute;
                    }
                })
                .or_insert((result, state.minute));
            }

            // done?
            if state.minute == limit {
                let result = state.pack.geode;
                insert(memo, &state, result);
                *max_result = (*max_result).max(result);
                return result;
            }

            // can we even beat max_result anymore?
            {
                // number of turns remaining.
                let remaining = (limit - state.minute) as u32;

                let max_yield =
                      // future yield of current geode bots.
                      remaining * state.pack.geode_robot as u32
                      // max future yield, if we build one geode bot
                      // on all future turns.
                    + remaining*(remaining-1)/2;

                if state.pack.geode as u32 + max_yield <= *max_result as u32 {
                    // doesn't matter what we insert,
                    // we already have a better result.
                    insert(memo, &state, 0);
                    return 0;
                }
            }

            let mut result = 0;

            if state.can_build_geode_robot(bp) {
                result = result.max(solution(state.step().build_geode_robot(bp), bp, limit, memo, max_result, true, true, true));
            }

            let mut new_can_obsidian = true;
            if state.can_build_obsidian_robot(bp) {
                new_can_obsidian = false;

                // can only build one bot per turn.
                // don't need more bots, if we're producing enough,
                // so we can build the most expensive bot on each turn.
                if can_obsidian && state.pack.obsidian_robot < bp.max_obsidian_cost() {
                    result = result.max(solution(state.step().build_obsidian_robot(bp), bp, limit, memo, max_result, true, true, true));
                }
            }

            let mut new_can_clay = true;
            if state.can_build_clay_robot(bp) {
                new_can_clay = false;

                if can_clay && state.pack.clay_robot < bp.max_clay_cost() {
                    result = result.max(solution(state.step().build_clay_robot(bp), bp, limit, memo, max_result, true, true, true));
                }
            }

            let mut new_can_ore = true;
            if state.can_build_ore_robot(bp) {
                new_can_ore = false;

                if can_ore && state.pack.ore_robot < bp.max_ore_cost() {
                    result = result.max(solution(state.step().build_ore_robot(bp), bp, limit, memo, max_result, true, true, true));
                }
            }

            result = result.max(solution(state.step(), bp, limit, memo, max_result, new_can_ore, new_can_clay, new_can_obsidian));

            insert(memo, &state, result);

            return result;
        }

        pub fn solve(bp: &Blueprint, limit: u8) -> u8 {
            let mut memo = HashMap::new();
            let mut max_result = 0;
            solution(State::new(), bp, limit, &mut memo, &mut max_result, true, true, true)
        }
    }

    pub mod v4 {
        use std::collections::HashMap;

        use super::{STATS, Blueprint, State};

        fn solution(state: State, bp: &Blueprint, limit: u8, memo: &mut HashMap<u64, (u8, u8)>, max_result: &mut u8,
            can_ore: bool, can_clay: bool, can_obsidian: bool
        ) -> u8 {
            STATS.with(|s| {
                s.memo_refs += 1;
                s.states_visited += 1;
            });

            let pack_64 = unsafe { core::mem::transmute(state.pack) };

            if let Some((result, minute)) = memo.get(&pack_64).copied() {
                if state.minute >= minute {
                    STATS.with(|s| { s.memo_hits += 1 });

                    *max_result = (*max_result).max(result);
                    return result;
                }
            }

            #[inline]
            fn insert(memo: &mut HashMap<u64, (u8, u8)>, state: &State, result: u8) {
                // note: for some reason, this is equivalent to simply overriding the result.
                //  ie: `memo.insert(pack, result)`
                //  could be, cause we're writing the results on the way up (going backwards in time).
                //  but not sure how that would work with "sibling branches".
                //  in any case, this isn't measurably slower, so who cares.
                memo.entry(unsafe { core::mem::transmute(state.pack) })
                .and_modify(|(old_result, old_minute)| {
                    if state.minute < *old_minute {
                        *old_result = result;
                        *old_minute = state.minute;
                    }
                })
                .or_insert((result, state.minute));
            }

            // done?
            if state.minute == limit {
                let result = state.pack.geode;
                insert(memo, &state, result);
                *max_result = (*max_result).max(result);
                return result;
            }

            // can we even beat max_result anymore?
            {
                // number of turns remaining.
                let remaining = (limit - state.minute) as u32;

                let max_yield =
                      // future yield of current geode bots.
                      remaining * state.pack.geode_robot as u32
                      // max future yield, if we build one geode bot
                      // on all future turns.
                    + remaining*(remaining-1)/2;

                if state.pack.geode as u32 + max_yield <= *max_result as u32 {
                    // doesn't matter what we insert,
                    // we already have a better result.
                    insert(memo, &state, 0);
                    return 0;
                }
            }

            let mut result = 0;

            // building a geode bot is the best thing we can do.
            // the proof is left as an exercise for the reader :P
            if state.can_build_geode_robot(bp) {
                result = result.max(solution(state.step().build_geode_robot(bp), bp, limit, memo, max_result, true, true, true));
            }
            else {
                let mut new_can_obsidian = true;
                if state.can_build_obsidian_robot(bp) {
                    new_can_obsidian = false;

                    // can only build one bot per turn.
                    // don't need more bots, if we're producing enough,
                    // so we can build the most expensive bot on each turn.
                    if can_obsidian && state.pack.obsidian_robot < bp.max_obsidian_cost() {
                        result = result.max(solution(state.step().build_obsidian_robot(bp), bp, limit, memo, max_result, true, true, true));
                    }
                }

                let mut new_can_clay = true;
                if state.can_build_clay_robot(bp) {
                    new_can_clay = false;

                    if can_clay && state.pack.clay_robot < bp.max_clay_cost() {
                        result = result.max(solution(state.step().build_clay_robot(bp), bp, limit, memo, max_result, true, true, true));
                    }
                }

                let mut new_can_ore = true;
                if state.can_build_ore_robot(bp) {
                    new_can_ore = false;

                    if can_ore && state.pack.ore_robot < bp.max_ore_cost() {
                        result = result.max(solution(state.step().build_ore_robot(bp), bp, limit, memo, max_result, true, true, true));
                    }
                }

                result = result.max(solution(state.step(), bp, limit, memo, max_result, new_can_ore, new_can_clay, new_can_obsidian));
            }

            insert(memo, &state, result);

            return result;
        }

        pub fn solve(bp: &Blueprint, limit: u8) -> u8 {
            let mut memo = HashMap::new();
            let mut max_result = 0;
            solution(State::new(), bp, limit, &mut memo, &mut max_result, true, true, true)
        }
    }

    pub mod v5 {
        use super::{STATS, Blueprint, State};

        fn solution(state: State, bp: &Blueprint, limit: u8, max_result: &mut u8,
            can_ore: bool, can_clay: bool, can_obsidian: bool
        ) {
            STATS.with(|s| {
                s.states_visited += 1;
            });

            // done?
            if state.minute == limit {
                let result = state.pack.geode;
                *max_result = (*max_result).max(result);
                return;
            }

            // can we even beat max_result anymore?
            {
                // number of turns remaining.
                let remaining = (limit - state.minute) as u32;

                let max_yield =
                      // future yield of current geode bots.
                      remaining * state.pack.geode_robot as u32
                      // max future yield, if we build one geode bot
                      // on all future turns.
                    + remaining*(remaining-1)/2;

                if state.pack.geode as u32 + max_yield <= *max_result as u32 {
                    return;
                }
            }

            // building a geode bot is the best thing we can do.
            // the proof is left as an exercise for the reader :P
            if state.can_build_geode_robot(bp) {
                solution(state.step().build_geode_robot(bp), bp, limit, max_result, true, true, true);
            }
            else {
                let mut new_can_obsidian = true;
                if state.can_build_obsidian_robot(bp) {
                    new_can_obsidian = false;

                    // can only build one bot per turn.
                    // don't need more bots, if we're producing enough,
                    // so we can build the most expensive bot on each turn.
                    if can_obsidian && state.pack.obsidian_robot < bp.max_obsidian_cost() {
                        solution(state.step().build_obsidian_robot(bp), bp, limit, max_result, true, true, true);
                    }
                }

                let mut new_can_clay = true;
                if state.can_build_clay_robot(bp) {
                    new_can_clay = false;

                    if can_clay && state.pack.clay_robot < bp.max_clay_cost() {
                        solution(state.step().build_clay_robot(bp), bp, limit, max_result, true, true, true);
                    }
                }

                let mut new_can_ore = true;
                if state.can_build_ore_robot(bp) {
                    new_can_ore = false;

                    if can_ore && state.pack.ore_robot < bp.max_ore_cost() {
                        solution(state.step().build_ore_robot(bp), bp, limit, max_result, true, true, true);
                    }
                }

                solution(state.step(), bp, limit, max_result, new_can_ore, new_can_clay, new_can_obsidian);
            }
        }

        pub fn solve(bp: &Blueprint, limit: u8) -> u8 {
            let mut max_result = 0;
            solution(State::new(), bp, limit, &mut max_result, true, true, true);
            max_result
        }
    }

    pub fn part_1<F: Fn(&Blueprint, u8) -> u8>(bps: &[Blueprint], f: F) {
        super::STATS.with(|s| s.reset());
        let t0 = std::time::Instant::now();
        let mut result = 0;
        for bp in bps {
            let geodes = f(bp, 24);
            // println!("geodes: {}", geodes);
            result += bp.id as u32 * geodes as u32;
        }
        println!("part 1 result: {} in {:?}", result, t0.elapsed());
        super::STATS.with(|s| s.print());
    }

    pub fn part_1_ex<F: Fn(&Blueprint, u8) -> u8>(bps: &[Blueprint], f: F, n: u8) {
        super::STATS.with(|s| s.reset());
        let t0 = std::time::Instant::now();
        let mut result = 0;
        for bp in bps {
            let geodes = f(bp, n);
            // println!("geodes: {}", geodes);
            result += bp.id as u32 * geodes as u32;
        }
        //println!("part 1 n: {}, result: {} in {:?}", n, result, t0.elapsed());
        println!("({}, {}), ", n, t0.elapsed().as_secs_f64());
        super::STATS.with(|s| s.print());
    }
}


pub fn main() {
    //let input = include_str!("d19-test.txt");
    let input = include_str!("d19-prod.txt");
    //let input = include_str!("d19-prod-2.txt");

    // baseline
    if 0==1 {
        use baseline::*;

        let input = parse(input);

        //printer::tree(&input[0], 5);

        //part_1(&input, survivor::solve);

        //println!("baseline");
        //part_1(case, v2::solve);
        println!("baseline");
        part_1(&input, v3::solve);
    }

    // pack
    if 0==1 {
        use pack::*;

        let input = parse(input);

        println!("dp earlier hit");
        part_1(&input, v1::solve);
    }

    // u8
    if 0==1 {
        use pack_u8::*;

        let input = parse(input);

        println!("u8");
        part_1(&input, v1::solve);
        println!("u8 as u64");
        part_1(&input, v2::solve);
    }

    // thonk
    if 1==1 {
        use thonk::*;

        let input = parse(input);

        println!("thonk max_result");
        part_1(&input, v1::solve);
        println!("thonk enough bots");
        part_1(&input, v2::solve);
        println!("thonk don't idle");
        part_1(&input, v3::solve);
        println!("thonk geodes first");
        part_1(&input, v4::solve);
        println!("thonk no memo");
        part_1(&input, v5::solve);

        for i in 10..100 {
            part_1_ex(&input, v5::solve, i);
        }
    }
}


