use ddsv::data;
use ddsv::data::{Process, Trans};
use env_logger;
use std::env;
use std::fmt;
use std::io::Write;

#[derive(Clone, PartialEq, Eq, Hash)]
struct SharedVars {
    mutex: i32,
    x: i32,
    t1: i32,
    t2: i32,
}

impl SharedVars {
    fn new() -> SharedVars {
        SharedVars {
            mutex: 0,
            x: 0,
            t1: 0,
            t2: 0,
        }
    }
}

impl fmt::Debug for SharedVars {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.write_fmt(format_args!(
            "m={} x={} t1={} t2={}",
            self.mutex, self.x, self.t1, self.t2
        ))
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
                vec![Trans::new(
                    String::from("lock"),
                    String::from("P1"),
                    is_locked,
                    lock,
                )],
            ),
            (
                String::from("P1"),
                vec![Trans::new(
                    String::from("read"),
                    String::from("P2"),
                    always_true,
                    move_x_to_t1,
                )],
            ),
            (
                String::from("P2"),
                vec![Trans::new(
                    String::from("inc"),
                    String::from("P3"),
                    always_true,
                    increment_t1,
                )],
            ),
            (
                String::from("P3"),
                vec![Trans::new(
                    String::from("write"),
                    String::from("P4"),
                    always_true,
                    move_t1_to_x,
                )],
            ),
            (
                String::from("P4"),
                vec![Trans::new(
                    String::from("unlock"),
                    String::from("P5"),
                    always_true,
                    unlock,
                )],
            ),
            (String::from("P5"), vec![]),
        ],
    };

    let process_q = Process {
        0: vec![
            (
                String::from("Q0"),
                vec![Trans::new(
                    String::from("lock"),
                    String::from("Q1"),
                    is_locked,
                    lock,
                )],
            ),
            (
                String::from("Q1"),
                vec![Trans::new(
                    String::from("read"),
                    String::from("Q2"),
                    always_true,
                    move_x_to_t2,
                )],
            ),
            (
                String::from("Q2"),
                vec![Trans::new(
                    String::from("inc"),
                    String::from("Q3"),
                    always_true,
                    increment_t2,
                )],
            ),
            (
                String::from("Q3"),
                vec![Trans::new(
                    String::from("write"),
                    String::from("Q4"),
                    always_true,
                    move_t2_to_x,
                )],
            ),
            (
                String::from("Q4"),
                vec![Trans::new(
                    String::from("unlock"),
                    String::from("Q5"),
                    always_true,
                    unlock,
                )],
            ),
            (String::from("Q5"), vec![]),
        ],
    };

    process_p.viz_process("m_inc2_1_P");
    process_q.viz_process("m_inc2_1_Q");
    let lts = data::concurrent_composition(&r0, &vec![process_p, process_q]);
    data::lts_print_deadlock(&lts);
    data::viz_lts("m_inc2_1", &lts);
}

fn always_true(_r: &SharedVars) -> bool {
    true
}

fn increment_t1(r: &SharedVars) -> SharedVars {
    let mut s = r.clone();
    s.t1 = r.t1 + 1;
    s
}

fn increment_t2(r: &SharedVars) -> SharedVars {
    let mut s = r.clone();
    s.t2 = r.t2 + 1;
    s
}
fn move_t1_to_x(r: &SharedVars) -> SharedVars {
    let mut s = r.clone();
    s.x = r.t1;
    s
}

fn move_t2_to_x(r: &SharedVars) -> SharedVars {
    let mut s = r.clone();
    s.x = r.t2;
    s
}
fn move_x_to_t1(r: &SharedVars) -> SharedVars {
    let mut s = r.clone();
    s.t1 = r.x;
    s
}
fn move_x_to_t2(r: &SharedVars) -> SharedVars {
    let mut s = r.clone();
    s.t2 = r.x;
    s
}
fn is_locked(r: &SharedVars) -> bool {
    r.mutex == 0
}

fn lock(r: &SharedVars) -> SharedVars {
    let mut s = r.clone();
    s.mutex = 1;
    s
}
fn unlock(r: &SharedVars) -> SharedVars {
    let mut s = r.clone();
    s.mutex = 0;
    s
}
