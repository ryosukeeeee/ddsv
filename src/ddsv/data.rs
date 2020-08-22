use log::debug;
use std::cmp::Eq;
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::fmt::Debug;
use std::fs;
use std::hash::Hash;
use std::io::{stdout, BufWriter, Write};
use std::process::Command;

use super::{Action, Guard, Label, Location, Path, State};

#[derive(Clone)]
pub struct Trans<T> {
    pub label: Label,
    pub location: Location,
    pub guard: Guard<T>,
    pub action: Action<T>,
}

impl<T> fmt::Debug for Trans<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.debug_struct("Trans")
            .field("label", &self.label)
            .field("location", &self.location)
            .finish()
    }
}

impl<T> Trans<T> {
    pub fn new(label: &str, location: &str, guard: Guard<T>, action: Action<T>) -> Trans<T> {
        Trans {
            label: String::from(label),
            location: String::from(location),
            guard: guard,
            action: action,
        }
    }
}

#[derive(Clone)]
pub struct Process<T>(pub Vec<(Location, Vec<Trans<T>>)>);

impl<T> fmt::Debug for Process<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.debug_list().entries(self.0.iter()).finish()
    }
}

impl<T> Process<T>
where
    T: Clone,
{
    pub fn new(v: Vec<(&str, Vec<Trans<T>>)>) -> Process<T> {
        let vv = v
            .iter()
            .map(move |(label, trans)| (String::from(*label), (*trans).clone()))
            .collect::<Vec<_>>();
        Process { 0: vv }
    }

    pub fn assoc(&self, location: &str) -> Option<&Vec<Trans<T>>> {
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

pub fn make_initial_state<T>(r0: &T, ps: &[Process<T>]) -> State<T>
where
    T: Clone,
{
    let v = ps.iter().map(|p| p.0[0].0.clone()).collect::<Vec<String>>();
    (r0.clone(), v)
}

pub fn calc_transitions<T>(
    acc: Vec<(Label, State<T>)>,
    r: &T,
    rs: &[Location],
    ls: &[Location],
    transitions: &[Trans<T>],
) -> Vec<(Label, State<T>)>
where
    T: Clone,
{
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
            acc__.insert(0, t);
            acc__
        } else {
            acc_
        }
    });
    tmp
}

pub fn collect_trans<T>(
    acc: Vec<(Label, State<T>)>,
    r: &T,
    rs: &[Location], // (sk-1, sk-2, ..., s2, s1)
    ls: &[Location], // (sk, sk+1, ..., sn)
    ps: &[Process<T>],
) -> Vec<(Label, State<T>)>
where
    T: Debug + Clone,
{
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

pub fn make_next_function<T>(ps: Vec<Process<T>>) -> Box<dyn Fn(State<T>) -> Vec<(Label, State<T>)>>
where
    T: Debug + Clone + 'static,
{
    Box::new(move |(r, locs)| return collect_trans(vec![], &r, &[], locs.as_slice(), ps.as_slice()))
}

pub fn concurrent_composition<T>(
    r0: &T,
    ps: &[Process<T>],
) -> (HashMap<State<T>, (i32, Path<T>)>, Vec<Path<T>>)
where
    T: Debug + Hash + Eq + Clone + 'static,
{
    let s0 = make_initial_state(r0, ps);
    let next = make_next_function(ps.to_vec());
    bfs(s0, next, "---")
}

pub fn bfs<T>(
    s0: State<T>,
    next: Box<dyn Fn(State<T>) -> Vec<(Label, State<T>)>>,
    label0: &str,
) -> (HashMap<State<T>, (i32, Path<T>)>, Vec<Path<T>>)
where
    T: Hash + Eq + Debug + Clone,
{
    let mut hm: HashMap<State<T>, (i32, Path<T>)> = HashMap::new();
    hm.insert(s0.clone(), (0, vec![]));

    let mut que: VecDeque<(State<T>, i32, Path<T>)> = VecDeque::new();
    que.push_front((s0.clone(), 0, vec![(label0.to_string(), s0.clone())]));
    let mut deadlocks: Vec<Path<T>> = vec![];

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

pub fn lts_print_deadlock<T>(lts: &(HashMap<State<T>, (i32, Path<T>)>, Vec<Path<T>>))
where
    T: Debug,
{
    let (_, deadlock) = lts;
    for dl in deadlock {
        println!("--------------------------------------");
        print_deadlock(dl);
    }
}

pub fn print_deadlock<T>(deadlock: &Path<T>)
where
    T: Debug,
{
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

pub fn viz_lts<T>(filename: &str, lts: &(HashMap<State<T>, (i32, Path<T>)>, Vec<Path<T>>))
where
    T: Debug + Hash + Eq,
{
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

fn emit_states<T>(ch: &mut dyn Write, hm: &HashMap<State<T>, (i32, Path<T>)>)
where
    T: Debug,
{
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

fn emit_transitions<T>(ch: &mut dyn Write, hm: &HashMap<State<T>, (i32, Path<T>)>)
where
    T: Debug + Hash + Eq,
{
    for (_key, (id, trans)) in hm.iter() {
        for (label, target) in trans {
            let (tid, _) = hm.get(target).unwrap();
            ch.write(format!("{} -> {} [label=\"{}\"];\n", id, tid, label).as_bytes())
                .unwrap();
        }
    }
}
