use log::debug;
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::fs;
use std::io::{BufWriter, Write};
use std::process::Command;

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

type Guard = Box<dyn Fn(&SharedVars) -> bool>;
type Action = Box<dyn Fn(&SharedVars) -> SharedVars>;
type Label = String;
type Location = String;
// type Trans = (label, location, guard, action);
// type Process = Vec<(Location, Vec<Trans>)>;
type State = Vec<Location>;

// #[derive(Clone)]
pub struct Trans {
    pub label: String,
    pub location: String,
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
pub struct Process(pub Vec<(String, Vec<Trans>)>);

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

pub fn make_initial_state(r0: &SharedVars, ps: &[Process]) -> (SharedVars, Vec<String>) {
    debug!("func make_initial_state is called");
    debug!("r0: {:?}", r0);
    let v = ps.iter().map(|p| p.0[0].0.clone()).collect::<Vec<String>>();
    (r0.clone(), v)
}

pub fn calc_transitions(
    acc: Vec<(String, (SharedVars, Vec<String>))>,
    r: &SharedVars,
    rs: &[String],
    ls: &[String],
    transitions: &[Trans],
) -> Vec<(String, (SharedVars, Vec<String>))> {
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
    acc: Vec<(String, (SharedVars, Vec<String>))>,
    r: &SharedVars,
    rs: &[String], // (sk-1, sk-2, ..., s2, s1)
    ls: &[String], // (sk, sk+1, ..., sn)
    ps: &[Process],
) -> Vec<(String, (SharedVars, Vec<String>))> {
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

pub fn make_next_function(
    ps: Vec<Process>,
) -> Box<dyn Fn(SharedVars, Vec<String>) -> Vec<(String, (SharedVars, Vec<String>))>> {
    debug!("func make_next_function is called");
    Box::new(move |r, locs| return collect_trans(vec![], &r, &[], locs.as_slice(), ps.as_slice()))
}
// acc: Vec<label, state> = Vec<String, (SharedVars, Vec<String>)>

// pub fn concurrent_composition(r0: SharedVars, ps: Vec<Vec<(String, Trans)>>) {
//     let s0 = make_initial_state(r0, ps);
//     let next = make_next_function(ps);
//     bfs(s0, next, "---")
// }

pub fn bfs(
    s0: (SharedVars, Vec<String>),
    next: Box<dyn Fn(SharedVars, Vec<String>) -> Vec<(String, (SharedVars, Vec<String>))>>,
    label0: &str,
) -> (
    HashMap<(SharedVars, Vec<String>), (i32, Vec<(String, (SharedVars, Vec<String>))>)>,
    Vec<Vec<(String, (SharedVars, Vec<String>))>>,
) {
    let mut hm: HashMap<
        (SharedVars, Vec<String>),
        (i32, Vec<(String, (SharedVars, Vec<String>))>),
    > = HashMap::new();
    hm.insert(s0.clone(), (0, vec![]));

    let mut que: VecDeque<(
        (SharedVars, Vec<String>),
        i32,
        Vec<(String, (SharedVars, Vec<String>))>,
    )> = VecDeque::new();
    que.push_front((s0.clone(), 0, vec![(label0.to_string(), s0.clone())]));
    let mut deadlocks: Vec<Vec<(String, (SharedVars, Vec<String>))>> = vec![];

    while !que.is_empty() {
        let (state, id, path) = que.pop_back().unwrap();
        let trans = (next)(state.clone().0, state.clone().1);
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

// state: (SharedVars, Vec<String>)
// path: Vec<(&str, (SharedVars, Vec<String>))>
// id: i32
// Que Item: (state, id, path)
// HashMap Item: HashMap<state, (i32, [])>
// next: Fn(SharedVars, Vec<String>) -> Vec<label, state> = Vec<String, (SharedVars, Vec<String>)>
