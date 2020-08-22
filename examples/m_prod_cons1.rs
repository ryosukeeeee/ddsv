use ddsv::data;
use ddsv::data::{Process, Trans};
use env_logger;
use std::env;
use std::fmt;
use std::io::Write;

#[derive(Clone, PartialEq, Eq, Hash)]
struct SharedVars {
    mutex: i32,
    cond: i32,
    count: i32,
}

static MAX_COUNT: i32 = 3;
static P_INDEX: i32 = 1;
static Q_INDEX: i32 = 2;
impl SharedVars {
    fn new() -> SharedVars {
        SharedVars {
            mutex: 0,
            cond: 0,
            count: 0,
        }
    }
}

impl fmt::Debug for SharedVars {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.write_fmt(format_args!(
            "m={} cv={} cnt={}",
            self.mutex, self.cond, self.count
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
                vec![Trans::new("lock", "P1", is_locked, lock)],
            ),
            (
                String::from("P1"),
                vec![
                    Trans::new("wait", "P2", can_wait_p, wait_p),
                    Trans::new("produce", "P3", can_produce, produce),
                ],
            ),
            (
                String::from("P2"),
                vec![Trans::new("wakeup", "P0", can_wakeup_p, wakeup)],
            ),
            (
                String::from("P3"),
                vec![Trans::new("signal", "P4", always_true, signal)],
            ),
            (
                String::from("P4"),
                vec![Trans::new("unlock", "P0", always_true, unlock)],
            ),
        ],
    };

    let process_q = Process {
        0: vec![
            (
                String::from("Q0"),
                vec![Trans::new("lock", "Q1", is_locked, lock)],
            ),
            (
                String::from("Q1"),
                vec![
                    Trans::new("wait", "Q2", can_wait_q, wait_q),
                    Trans::new("consume", "Q3", can_consume, consume),
                ],
            ),
            (
                String::from("Q2"),
                vec![Trans::new("wakeup", "Q0", can_wakeup_q, wakeup)],
            ),
            (
                String::from("Q3"),
                vec![Trans::new("signal", "Q4", always_true, signal)],
            ),
            (
                String::from("Q4"),
                vec![Trans::new("unlock", "Q0", always_true, unlock)],
            ),
        ],
    };

    process_p.viz_process("m_prod_cons1_P");
    process_q.viz_process("m_prod_cons1_Q");
    let lts = data::concurrent_composition(&r0, &vec![process_p, process_q]);
    data::lts_print_deadlock(&lts);
    data::viz_lts("m_prod_cons1", &lts);
}

// guard
fn always_true(_r: &SharedVars) -> bool {
    true
}

fn is_locked(r: &SharedVars) -> bool {
    r.mutex == 0
}

fn can_wait_p(r: &SharedVars) -> bool {
    r.count == MAX_COUNT
}

fn can_wait_q(r: &SharedVars) -> bool {
    r.count == 0
}

fn can_produce(r: &SharedVars) -> bool {
    r.count < MAX_COUNT
}

fn can_consume(r: &SharedVars) -> bool {
    r.count > 0
}

fn can_wakeup_p(r: &SharedVars) -> bool {
    (r.cond & P_INDEX) == 0
}

fn can_wakeup_q(r: &SharedVars) -> bool {
    (r.cond & Q_INDEX) == 0
}

// action
fn lock(r: &SharedVars) -> SharedVars {
    let mut s = r.clone();
    s.mutex = 1;
    s
}

fn wait_p(r: &SharedVars) -> SharedVars {
    let mut s = r.clone();
    s.mutex = 0;
    s.cond = r.cond | P_INDEX;
    s
}

fn wait_q(r: &SharedVars) -> SharedVars {
    let mut s = r.clone();
    s.mutex = 0;
    s.cond = r.cond | Q_INDEX;
    s
}

fn produce(r: &SharedVars) -> SharedVars {
    let mut s = r.clone();
    s.count = r.count + 1;
    s
}

fn consume(r: &SharedVars) -> SharedVars {
    let mut s = r.clone();
    s.count = r.count - 1;
    s
}
fn wakeup(r: &SharedVars) -> SharedVars {
    r.clone()
}

fn signal(r: &SharedVars) -> SharedVars {
    let mut s = r.clone();
    s.cond = r.cond & (r.cond - 1);
    s
}

fn unlock(r: &SharedVars) -> SharedVars {
    let mut s = r.clone();
    s.mutex = 0;
    s
}
