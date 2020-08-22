use ddsv::data;
use ddsv::data::{Process, Trans};
use env_logger;
use std::env;
use std::fmt;
use std::io::Write;

#[derive(Clone, PartialEq, Eq, Hash)]
struct SharedVars {
    m0: i32,
    m1: i32,
}

impl SharedVars {
    fn new() -> SharedVars {
        SharedVars { m0: 0, m1: 0 }
    }
}

impl fmt::Debug for SharedVars {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.write_fmt(format_args!("m0={} m1={}", self.m0, self.m1))
    }
}

fn main() {
    env::set_var("RUST_LOG", "info");
    env_logger::builder()
        .format(|buf, record| {
            writeln!(
                buf,
                "{}: L{} {}",
                record.level(),
                record.line().unwrap_or(0),
                record.args()
            )
        })
        .init();
    let r0 = SharedVars::new();
    let process_p = Process {
        0: vec![
            (
                String::from("P0"),
                vec![Trans::new("lock 0", "P1", is_locked_0, lock_0)],
            ),
            (
                String::from("P1"),
                vec![Trans::new("lock 1", "P2", is_locked_1, lock_1)],
            ),
            (
                String::from("P2"),
                vec![Trans::new("unlock 1", "P3", always_true, unlock_1)],
            ),
            (
                String::from("P3"),
                vec![Trans::new("unlock 0", "P0", always_true, unlock_0)],
            ),
        ],
    };

    let process_q = Process {
        0: vec![
            (
                String::from("Q0"),
                vec![Trans::new("lock 1", "Q1", is_locked_1, lock_1)],
            ),
            (
                String::from("Q1"),
                vec![Trans::new("lock 0", "Q2", is_locked_0, lock_0)],
            ),
            (
                String::from("Q2"),
                vec![Trans::new("unlock 0", "Q3", always_true, unlock_0)],
            ),
            (
                String::from("Q3"),
                vec![Trans::new("unlock 1", "Q0", always_true, unlock_1)],
            ),
        ],
    };
    process_p.viz_process("m_mutex2_P");
    process_q.viz_process("m_mutex2_Q");
    let lts = data::concurrent_composition(&r0, &vec![process_p, process_q]);
    data::lts_print_deadlock(&lts);
    data::viz_lts("m_mutex2", &lts);
}

// guard
fn is_locked_0(r: &SharedVars) -> bool {
    r.m0 == 0
}

fn is_locked_1(r: &SharedVars) -> bool {
    r.m1 == 0
}

fn always_true(_r: &SharedVars) -> bool {
    true
}

// action
fn lock_0(r: &SharedVars) -> SharedVars {
    let mut s = r.clone();
    s.m0 = 1;
    s
}

fn lock_1(r: &SharedVars) -> SharedVars {
    let mut s = r.clone();
    s.m1 = 1;
    s
}

fn unlock_0(r: &SharedVars) -> SharedVars {
    let mut s = r.clone();
    s.m0 = 0;
    s
}

fn unlock_1(r: &SharedVars) -> SharedVars {
    let mut s = r.clone();
    s.m1 = 0;
    s
}
