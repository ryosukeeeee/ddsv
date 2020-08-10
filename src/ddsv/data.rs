use std::fmt;
use std::fs;
use std::io::{BufWriter, Write};
use std::process::Command;

#[derive(Debug, Clone)]
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

pub struct Trans {
    pub label: String,
    pub location: String,
    pub guard: Box<dyn Fn(SharedVars) -> bool>,
    pub action: Box<dyn Fn(SharedVars) -> SharedVars>,
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
    pub fn new(
        label: String,
        location: String,
        guard: Box<dyn Fn(SharedVars) -> bool>,
        action: Box<dyn Fn(SharedVars) -> SharedVars>,
    ) -> Trans {
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

pub struct Process(pub Vec<(String, Vec<Trans>)>);

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

pub fn make_initial_state(r0: &SharedVars, ps: &Vec<Process>) -> Vec<(SharedVars, String)> {
    ps.iter()
        .map(|p| (r0.clone(), p.0[0].0.clone()))
        .collect::<Vec<(SharedVars, String)>>()
}

// pub fn calc_transition(
//     acc: Vec<(String, String)>,
//     r: SharedVars,
//     rs: Vec<String>,
//     ls: Vec<String>,
//     transitions: Vec<Trans>,
// ) {
//     transitions
//         .iter()
//         .fold(acc, |acc, (label, target, guard, action)| {
//             if (guard)(r) {
//                 let locations =
//                 let target = ((action)(r), locations);
//                 let t = (label, target);
//                 acc.insert(0, t);
//                 acc
//             } else {
//                 acc
//             }
//         })
// }

// pub fn collect_trans(
//     acc: Vec<(String, String)>,
//     r: SharedVars,
//     rs: Vec<String>,
//     ls: Vec<String>,
//     ps: Vec<Process>,
// ) -> Vec<(String, String)> {
//     match (ls, ps) {
//         (vec![], vec![]) => acc,
//         (l, p) => {
//             let (location, ls_2) = l.split_first().unwrap();
//             let (process, ps_2) = p.split_first().unwrap();
//             // p_first
//             let transitions = process.assoc(&location);
//             let acc_ = calc_transition(acc, r, rs, ls_2.to_vec(), transitions);
//             collect_trans(acc, r, rs, ls_2.to_vec(), ps_2.to_vec());
//             vec![]
//         }
//     }
// }

// pub fn make_next_function(ps: Vec<Process>) {}

// pub fn concurrent_composition(r0: SharedVars, ps: Vec<Vec<(String, Trans)>>) {
//     let s0 = make_initial_state(r0, ps);
//     let next = make_next_function(ps);
//     bfs(s0, next, "---")
// }
