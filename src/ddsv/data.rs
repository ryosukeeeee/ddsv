use log::debug;
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::fs;
use std::io::{BufWriter, Write};
use std::process::Command;

use super::{Action, Guard, Label, Location, Path, State};

#[derive(Debug, Clone, Eq, Hash)]
pub struct SharedVars {
    pub x: i32,
    pub t1: i32,
    pub t2: i32,
}
impl SharedVars {
    pub fn new() -> SharedVars {
        SharedVars { x: 0, t1: 0, t2: 0 }
    }
}

// #[derive(Clone)]
pub struct Trans {
    pub label: Label,
    pub location: Location,
    pub guard: Guard,
    pub action: Action,
}

impl fmt::Debug for Trans {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.debug_struct("Trans")
            .field("label", &self.label)
            .field("location", &self.location)
            .finish()
    }
}

impl Trans {
    pub fn new(label: String, location: String, guard: Guard, action: Action) -> Trans {
        Trans {
            label: label,
            location: location,
            guard: guard,
            action: action,
        }
    }
}

impl PartialEq for SharedVars {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.t1 == other.t1 && self.t2 == other.t2
    }
}

// #[derive(Debug)]
pub struct Process(pub Vec<(Location, Vec<Trans>)>);

impl fmt::Debug for Process {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.debug_list().entries(self.0.iter()).finish()
    }
}

impl Process {
    pub fn assoc(&self, location: &str) -> Option<&Vec<Trans>> {
        for v in &self.0 {
            if v.0 == location {
                return Some(v.1.as_ref());
            }
        }
        None
    }
    pub fn viz_process(&self, filename: &str) {
        let mut f = BufWriter::new(fs::File::create(format!("{}.dot", filename)).unwrap());
        f.write("digraph {\n".as_bytes()).unwrap();

        &self
            .0
            .iter()
            .map(|v| {
                f.write(format!("{};\n", v.0.to_string()).as_bytes())
                    .unwrap();
            })
            .collect::<Vec<_>>();

        &self
            .0
            .iter()
            .map(|v| {
                v.1.iter().for_each(|trans| {
                    let target = &trans.location;
                    let label = &trans.label;
                    let line = format!("{} -> {} [label=\"{}\"];\n", &v.0, &target, &label);
                    f.write(&line.as_bytes()).unwrap();
                });
            })
            .collect::<Vec<_>>();

        f.write("}\n".as_bytes()).unwrap();

        Command::new("dot")
            .arg("-T")
            .arg("pdf")
            .arg("-o")
            .arg(format!("{}.pdf", filename))
            .arg(format!("{}.dot", filename))
            .spawn()
            .expect("failed to visualize");
    }
}

pub fn make_initial_state(r0: &SharedVars, ps: &[Process]) -> State {
    debug!("func make_initial_state is called");
    debug!("r0: {:?}", r0);
    let v = ps.iter().map(|p| p.0[0].0.clone()).collect::<Vec<String>>();
    (r0.clone(), v)
}

pub fn calc_transitions(
    acc: Vec<(Label, State)>,
    r: &SharedVars,
    rs: &[String],
    ls: &[String],
    transitions: &[Trans],
) -> Vec<(Label, State)> {
    let tmp = transitions.iter().fold(acc, |acc_, trans| {
        if (trans.guard)(&r) {
            // guardが成立 => 遷移可能
            let label = &trans.label; // label = "read"
            let mut v1 = ls.to_vec(); // ls = (sk, sk+1, ..., sn)
            v1.insert(0, trans.location.clone()); // location = P1

            let mut locations = rs.to_vec(); // rs = (sk-1, sk-2, ..., s2, s1)
            locations.reverse();
            locations.append(&mut v1);
            debug!(
                "  rs: {:?},\n  ls: {:?},\n  locations: {:?}",
                rs, ls, locations
            );
            // target = (遷移後の共有変数, (s1, s2, ..., sn))
            let target = ((trans.action)(&r), locations);
            // t = ("read", (遷移後の共有変数, (s1, s2, ..., sn)))
            let t = (String::from(label), target);
            let mut acc__ = acc_.clone();
            acc__.push(t);
            acc__
        } else {
            acc_
        }
    });
    tmp
}

pub fn collect_trans(
    acc: Vec<(Label, State)>,
    r: &SharedVars,
    rs: &[String], // (sk-1, sk-2, ..., s2, s1)
    ls: &[String], // (sk, sk+1, ..., sn)
    ps: &[Process],
) -> Vec<(Label, State)> {
    match (ls, ps) {
        ([], []) => {
            debug!("empty");
            debug!("ps: {:?}", ps);
            debug!("ls: {:?}", ls);
            acc
        }
        (l, p) => {
            debug!("ps: {:?}", ps);
            debug!("ls: {:?}", ls);
            let (location, ls_2) = l.split_first().unwrap();
            let (process, ps_2) = p.split_first().unwrap();
            if let Some(transitions) = process.assoc(&location) {
                let acc = calc_transitions(acc, r, rs, ls_2, transitions);
                let mut rs_2 = vec![location.clone()];
                rs_2.extend(rs.to_vec());
                return collect_trans(acc, r, rs_2.as_slice(), ls_2, ps_2);
            } else {
                return vec![];
            }
        }
    }
}

pub fn make_next_function(ps: Vec<Process>) -> Box<dyn Fn(State) -> Vec<(Label, State)>> {
    debug!("func make_next_function is called");
    Box::new(move |(r, locs)| return collect_trans(vec![], &r, &[], locs.as_slice(), ps.as_slice()))
}

// pub fn concurrent_composition(r0: SharedVars, ps: Vec<Vec<(String, Trans)>>) {
//     let s0 = make_initial_state(r0, ps);
//     let next = make_next_function(ps);
//     bfs(s0, next, "---")
// }

pub fn bfs(
    s0: State,
    next: Box<dyn Fn(State) -> Vec<(Label, State)>>,
    label0: &str,
) -> (HashMap<State, (i32, Path)>, Vec<Path>) {
    let mut hm: HashMap<State, (i32, Path)> = HashMap::new();
    hm.insert(s0.clone(), (0, vec![]));

    let mut que: VecDeque<(State, i32, Path)> = VecDeque::new();
    que.push_front((s0.clone(), 0, vec![(label0.to_string(), s0.clone())]));
    let mut deadlocks: Vec<Path> = vec![];

    while !que.is_empty() {
        let (state, id, path) = que.pop_back().unwrap();
        let trans = (next)(state.clone());
        if trans.is_empty() {
            deadlocks.push(path.clone());
        }

        hm.insert(state, (id, trans.clone()));
        for (label, target) in &trans {
            if !hm.contains_key(target) {
                let id = hm.len() as i32;
                hm.insert(target.clone(), (id, vec![]));
                let mut v = path.clone();
                v.push((label.clone(), target.clone()));
                que.push_front((target.clone(), id, v));
            }
        }
    }
    (hm, deadlocks)
}

pub fn lts_print_deadlock(lts: (HashMap<State, (i32, Path)>, Vec<Path>)) {
    let (_, deadlock) = lts;
    for dl in deadlock {
        println!("--------------------------------------");
        print_deadlock(dl);
    }
}

pub fn print_deadlock(deadlock: Path) {
    for (i, dl) in deadlock.iter().enumerate() {
        print!("{} {:010} {:?} ", i, dl.0, (dl.1).0);
        print_locations((dl.1).1.as_slice());
        print!("\n");
    }
}

pub fn print_locations(locations: &[Location]) {
    for l in locations {
        print!("{} ", l)
    }
}
