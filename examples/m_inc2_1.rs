use ddsv::data;
use ddsv::data::Visualize;
use ddsv::ddsv::Process;
use ddsv::process;
use std::env;
use std::fmt;
use std::io::Write;
use std::rc::Rc;

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
    let process_p: Process<SharedVars> = process![
        SharedVars,
        [
            (
                "P0",
                [(
                    "lock",
                    "P1",
                    |r: &SharedVars| r.mutex == 0,
                    |r: &SharedVars| {
                        let mut s = r.clone();
                        s.mutex = 1;
                        s
                    }
                )]
            ),
            (
                "P1",
                [(
                    "read",
                    "P2",
                    |_: &SharedVars| true,
                    |r: &SharedVars| {
                        let mut s = r.clone();
                        s.t1 = r.x;
                        s
                    }
                )]
            ),
            (
                "P2",
                [(
                    "inc",
                    "P3",
                    |_: &SharedVars| true,
                    |r: &SharedVars| {
                        let mut s = r.clone();
                        s.t1 = r.t1 + 1;
                        s
                    }
                )]
            ),
            (
                "P3",
                [(
                    "write",
                    "P4",
                    |_: &SharedVars| true,
                    |r: &SharedVars| {
                        let mut s = r.clone();
                        s.x = r.t1;
                        s
                    }
                )]
            ),
            (
                "P4",
                [(
                    "unlock",
                    "P5",
                    |_: &SharedVars| true,
                    |r: &SharedVars| {
                        let mut s = r.clone();
                        s.mutex = 0;
                        s
                    }
                )]
            ),
            ("P5", [])
        ]
    ];

    let process_q: Process<SharedVars> = process![
        SharedVars,
        [
            (
                "Q0",
                [(
                    "lock",
                    "Q1",
                    |r: &SharedVars| r.mutex == 0,
                    |r: &SharedVars| {
                        let mut s = r.clone();
                        s.mutex = 1;
                        s
                    }
                )]
            ),
            (
                "Q1",
                [(
                    "read",
                    "Q2",
                    |_: &SharedVars| true,
                    |r: &SharedVars| {
                        let mut s = r.clone();
                        s.t2 = r.x;
                        s
                    }
                )]
            ),
            (
                "Q2",
                [(
                    "inc",
                    "Q3",
                    |_: &SharedVars| true,
                    |r: &SharedVars| {
                        let mut s = r.clone();
                        s.t2 = r.t2 + 1;
                        s
                    }
                )]
            ),
            (
                "Q3",
                [(
                    "write",
                    "Q4",
                    |_: &SharedVars| true,
                    |r: &SharedVars| {
                        let mut s = r.clone();
                        s.x = r.t2;
                        s
                    }
                )]
            ),
            (
                "Q4",
                [(
                    "unlock",
                    "Q5",
                    |_: &SharedVars| true,
                    |r: &SharedVars| {
                        let mut s = r.clone();
                        s.mutex = 0;
                        s
                    }
                )]
            ),
            ("Q5", [])
        ]
    ];

    process_p.viz_process("m_inc2_1_P");
    process_q.viz_process("m_inc2_1_Q");
    let lts = data::concurrent_composition(r0, &[process_p, process_q]);
    data::lts_print_deadlock(&lts);
    data::viz_lts("m_inc2_1", &lts);
}
