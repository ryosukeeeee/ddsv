use log::debug;
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::fs;
use std::io::{stdout, BufWriter, Write};
use std::process::Command;

use super::{Action, Guard, Label, Location, Path, State};

#[derive(Clone, Eq, Hash)]
pub struct SharedVars {
    pub x: i32,
    pub t1: i32,
    pub t2: i32,
}
impl fmt::Debug for SharedVars {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.write_fmt(format_args!("x={} t1={} t2={}", self.x, self.t1, self.t2))
    }
}
impl SharedVars {
    pub fn new() -> SharedVars {
        SharedVars { x: 0, t1: 0, t2: 0 }
    }
}

#[derive(Clone)]
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

#[derive(Clone)]
pub struct Process(pub Vec<(Location, Vec<Trans>)>);

impl fmt::Debug for Process {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.debug_list().entries(self.0.iter()).finish()
    }
}

impl Process {
    pub fn assoc(&self, location: &str) -> Option<&Vec<Trans>> {
        debug!("location: {}", location);
        for v in &self.0 {
            debug!("assoc: {:?}", v);
            if v.0 == location {
                debug!("  found: {:?}", v);
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
    let v = ps.iter().map(|p| p.0[0].0.clone()).collect::<Vec<String>>();
    (r0.clone(), v)
}

pub fn calc_transitions(
    acc: Vec<(Label, State)>,
    r: &SharedVars,
    rs: &[Location],
    ls: &[Location],
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
    rs: &[Location], // (sk-1, sk-2, ..., s2, s1)
    ls: &[Location], // (sk, sk+1, ..., sn)
    ps: &[Process],
) -> Vec<(Label, State)> {
    match (ls, ps) {
        ([], []) => {
            debug!("collect_trans: {:?}", acc);
            acc
        }
        (l, p) => {
            debug!("ps: {:?}", ps);
            debug!("ls: {:?}", ls);
            let (location, ls_2) = l.split_first().unwrap();
            let (process, ps_2) = p.split_first().unwrap();
            let transitions = process.assoc(&location).unwrap();
            let acc = calc_transitions(acc, r, rs, ls_2, transitions);
            let mut rs_2 = vec![location.clone()];
            rs_2.extend(rs.to_vec());
            return collect_trans(acc, r, rs_2.as_slice(), ls_2, ps_2);
        }
    }
}

pub fn make_next_function(ps: Vec<Process>) -> Box<dyn Fn(State) -> Vec<(Label, State)>> {
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
        debug!("==================");
        debug!("state: {:?}, id: {}, \npath: {:?}", state, id, path);
        debug!("==================");
        let trans = (next)(state.clone());
        // trans.reverse();
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

pub fn lts_print_deadlock(lts: &(HashMap<State, (i32, Path)>, Vec<Path>)) {
    let (_, deadlock) = lts;
    for dl in deadlock {
        println!("--------------------------------------");
        print_deadlock(dl);
    }
}

pub fn print_deadlock(deadlock: &Path) {
    let out = stdout();
    let mut out = BufWriter::new(out.lock());
    for (i, dl) in deadlock.iter().enumerate() {
        out.write(format!("{} {:010} {:?} ", i, dl.0, (dl.1).0).as_bytes())
            .unwrap();
        print_locations(&mut out, (dl.1).1.as_slice());
        out.write("\n".as_bytes()).unwrap();
    }
}

pub fn print_locations(ch: &mut dyn Write, locations: &[Location]) {
    for l in locations {
        ch.write(format!("{} ", l).as_bytes()).unwrap();
    }
}

pub fn viz_lts(filename: &str, lts: &(HashMap<State, (i32, Path)>, Vec<Path>)) {
    let (ht, _) = lts;
    let mut f = BufWriter::new(fs::File::create(format!("{}.dot", filename)).unwrap());
    f.write("digraph{\n".as_bytes()).unwrap();
    emit_states(&mut f, &ht);
    emit_transitions(&mut f, &ht);
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

fn emit_states(ch: &mut dyn Write, hm: &HashMap<State, (i32, Path)>) {
    for ((r, locs), (id, trans)) in hm.iter() {
        ch.write(format!("{} [label=\"{}\\n", id, id).as_bytes())
            .unwrap();
        print_locations(ch, locs);
        ch.write(format!("\\n{:?}\",", r).as_bytes()).unwrap();
        if *id == 0 {
            ch.write("style=filled,fillcolor=cyan".as_bytes()).unwrap();
        } else if trans.len() == 0 {
            ch.write("style=filled,fillcolor=pink".as_bytes()).unwrap();
        }
        ch.write("];\n".as_bytes()).unwrap();
    }
}

fn emit_transitions(ch: &mut dyn Write, hm: &HashMap<State, (i32, Path)>) {
    for (_key, (id, trans)) in hm.iter() {
        for (label, target) in trans {
            let (tid, _) = hm.get(target).unwrap();
            ch.write(format!("{} -> {} [label=\"{}\"];\n", id, tid, label).as_bytes())
                .unwrap();
        }
    }
}
