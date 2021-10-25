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
    x: i32,
    t1: i32,
    t2: i32,
}

impl SharedVars {
    fn new() -> SharedVars {
        SharedVars { x: 0, t1: 0, t2: 0 }
    }
}

impl fmt::Debug for SharedVars {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.write_fmt(format_args!("x={} t1={} t2={}", self.x, self.t1, self.t2))
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
                    "read",
                    "P1",
                    |_: &SharedVars| true,
                    |r: &SharedVars| {
                        let mut s = r.clone();
                        s.t1 = r.x;
                        s
                    }
                )]
            ),
            (
                "P1",
                [(
                    "inc",
                    "P2",
                    |_: &SharedVars| true,
                    |r: &SharedVars| {
                        let mut s = r.clone();
                        s.t1 = r.t1 + 1;
                        s
                    }
                )]
            ),
            (
                "P2",
                [(
                    "write",
                    "P3",
                    |_: &SharedVars| true,
                    |r: &SharedVars| {
                        let mut s = r.clone();
                        s.x = r.t1;
                        s
                    }
                )]
            ),
            ("P3", [])
        ]
    ];

    let process_q: Process<SharedVars> = process![
        SharedVars,
        [
            (
                "Q0",
                [(
                    "read",
                    "Q1",
                    |_: &SharedVars| true,
                    |r: &SharedVars| {
                        let mut s = r.clone();
                        s.t2 = r.x;
                        s
                    }
                )]
            ),
            (
                "Q1",
                [(
                    "inc",
                    "Q2",
                    |_: &SharedVars| true,
                    |r: &SharedVars| {
                        let mut s = r.clone();
                        s.t2 = r.t2 + 1;
                        s
                    }
                )]
            ),
            (
                "Q2",
                [(
                    "write",
                    "Q3",
                    |_: &SharedVars| true,
                    |r: &SharedVars| {
                        let mut s = r.clone();
                        s.x = r.t2;
                        s
                    }
                )]
            ),
            ("Q3", [])
        ]
    ];
    process_p.viz_process("m_inc2_P");
    process_q.viz_process("m_inc2_Q");
    let lts = data::concurrent_composition(r0, &[process_p, process_q]);
    data::lts_print_deadlock(&lts);
    data::viz_lts("m_inc2", &lts);
}
