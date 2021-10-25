use log::debug;
use std::cmp::Eq;
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::fmt::Debug;
use std::fs;
use std::hash::Hash;
use std::io::{stdout, BufWriter, Write};
use std::process::Command;

use super::{Label, Location, Path, Process, State, Trans};

impl<T> fmt::Debug for Trans<T>
where
    T: Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.debug_struct("Trans")
            .field("label", &self.label)
            .field("location", &self.location)
            .finish()
    }
}

pub trait Visualize<T: Clone> {
    fn assoc(&self, location: &str) -> Option<&Vec<Trans<T>>>;
    fn viz_process(&self, filename: &str);
}

impl<T> Visualize<T> for Process<T>
where
    T: Clone,
{
    // ラベル（location）を受け取り、状態遷移の配列から、
    // ラベルが一致する遷移の配列があったら、それへの参照を返す。
    // ラベルが一致する遷移の配列がなければNoneを返す
    fn assoc(&self, location: &str) -> Option<&Vec<Trans<T>>> {
        for v in self {
            if v.0 == location {
                return Some(v.1.as_ref());
            }
        }
        None
    }

    fn viz_process(&self, filename: &str) {
        let mut f = BufWriter::new(fs::File::create(format!("{}.dot", filename)).unwrap());
        f.write_all("digraph {\n".as_bytes()).unwrap();

        for p in self {
            f.write_all(format!("{};\n", p.0.to_string()).as_bytes())
                .unwrap();
        }

        for p in self {
            p.1.iter().for_each(|trans| {
                let target = &trans.location;
                let label = &trans.label;
                let line = format!("{} -> {} [label=\"{}\"];\n", &p.0, &target, &label);
                f.write_all(&line.as_bytes()).unwrap();
            });
        }

        f.write_all("}\n".as_bytes()).unwrap();

        Command::new("dot")
            .arg("-T")
            .arg("pdf")
            .arg("-o")
            .arg(format!("{}.pdf", filename))
            .arg(format!("{}.dot", filename))
            .spawn()
            .expect("failed to visualize");
        debug!("succeed to output PDF");
    }
}

// 最初の共有変数と、プロセスのリストを受け取って状態を返す
pub fn make_initial_state<T>(r0: T, ps: &[Process<T>]) -> State<T>
where
    T: Clone,
{
    let mut v = Vec::new();
    for process in ps {
        let first = process.first().expect("Empty process is prohobited");
        v.push(first.0.clone())
    }
    (r0, v)
}

pub fn collect_trans<T>(
    r: &T,             // 共有変数
    locs: &[Location], // ex. [P1, Q0]
    ps: &[Process<T>], // プロセスの配列
) -> Vec<(Label, State<T>)>
where
    T: Debug + Clone,
{
    // lsが未処理、初期値は空リスト
    // rsが処理済み、初期値は各プロセスの
    let mut result: Vec<(Label, State<T>)> = Vec::new();
    for i in 0..locs.len() {
        let process = &ps[i];
        let location = &locs[i];

        let transitions = process.assoc(location).unwrap();
        let out: Vec<(Label, Location, T)> = calc_transitions(r, transitions);
        for tuple in out {
            let mut locations = locs.to_vec();
            locations[i] = tuple.1;
            result.push((tuple.0, (tuple.2, locations)));
        }
    }
    result
}

pub fn calc_transitions<T>(r: &T, transitions: &[Trans<T>]) -> Vec<(Label, Location, T)>
where
    T: Clone,
{
    let mut result: Vec<(Label, Location, T)> = Vec::new();
    for trans in transitions {
        if (trans.guard)(&r) {
            let label = trans.label.clone(); // ex. "read"
            let location = trans.location.clone(); // ex. "P1"
            let r_transed = (trans.action)(&r); // 遷移後の共有変数
            result.push((label, location, r_transed));
        }
    }
    result
}
// 状態（共有変数と各スレッドの状態）を渡すと、遷移可能なラベル（ex. read）とそのときの遷移先の状態を配列にして返す関数を計算する関数
pub fn make_next_function<T>(ps: Vec<Process<T>>) -> Box<dyn Fn(State<T>) -> Vec<(Label, State<T>)>>
where
    T: Debug + Clone + 'static,
{
    Box::new(move |(r, locs)| collect_trans(&r, &locs, &ps))
}

pub fn concurrent_composition<T>(
    r0: T,
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
        out.write_all(format!("{} {:010} {:?} ", i, dl.0, (dl.1).0).as_bytes())
            .unwrap();
        print_locations(&mut out, (dl.1).1.as_slice());
        out.write_all("\n".as_bytes()).unwrap();
    }
}

pub fn print_locations(ch: &mut dyn Write, locations: &[Location]) {
    for l in locations {
        ch.write_all(format!("{} ", l).as_bytes()).unwrap();
    }
}

pub fn viz_lts<T>(filename: &str, lts: &(HashMap<State<T>, (i32, Path<T>)>, Vec<Path<T>>))
where
    T: Debug + Hash + Eq,
{
    let (ht, _) = lts;
    let mut f = BufWriter::new(fs::File::create(format!("{}.dot", filename)).unwrap());
    f.write_all("digraph{\n".as_bytes()).unwrap();
    emit_states(&mut f, &ht);
    emit_transitions(&mut f, &ht);
    f.write_all("}\n".as_bytes()).unwrap();

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
        ch.write_all(format!("{} [label=\"{}\\n", id, id).as_bytes())
            .unwrap();
        print_locations(ch, locs);
        ch.write_all(format!("\\n{:?}\",", r).as_bytes()).unwrap();
        if *id == 0 {
            ch.write_all("style=filled,fillcolor=cyan".as_bytes())
                .unwrap();
        } else if trans.is_empty() {
            ch.write_all("style=filled,fillcolor=pink".as_bytes())
                .unwrap();
        }
        ch.write_all("];\n".as_bytes()).unwrap();
    }
}

fn emit_transitions<T>(ch: &mut dyn Write, hm: &HashMap<State<T>, (i32, Path<T>)>)
where
    T: Debug + Hash + Eq,
{
    for (_key, (id, trans)) in hm.iter() {
        for (label, target) in trans {
            let (tid, _) = hm.get(target).unwrap();
            ch.write_all(format!("{} -> {} [label=\"{}\"];\n", id, tid, label).as_bytes())
                .unwrap();
        }
    }
}
