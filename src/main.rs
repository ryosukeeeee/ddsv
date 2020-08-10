mod ddsv;

use ddsv::data::{Process, SharedVars, Trans};

fn main() {
    // let r0 = SharedVars { x: 0, t1: 0, t2: 0 };
    let process_p = Process {
        0: vec![
            (
                String::from("P0"),
                vec![Trans::new(
                    String::from("read"),
                    String::from("P1"),
                    Box::new(|_r| true),
                    Box::new(|r| {
                        let mut s = r.clone();
                        s.t1 = r.x;
                        s
                    }),
                )],
            ),
            (
                String::from("P1"),
                vec![Trans::new(
                    String::from("inc"),
                    String::from("P2"),
                    Box::new(|_r| true),
                    Box::new(|r| {
                        let mut s = r.clone();
                        s.t1 = r.t1 + 1;
                        s
                    }),
                )],
            ),
            (
                String::from("P2"),
                vec![Trans::new(
                    String::from("write"),
                    String::from("P3"),
                    Box::new(|_r| true),
                    Box::new(|r| {
                        let mut s = r.clone();
                        s.x = r.t1;
                        s
                    }),
                )],
            ),
            (String::from("P3"), vec![]),
        ],
    };

    let process_q = Process {
        0: vec![
            (
                String::from("Q0"),
                vec![Trans::new(
                    String::from("read"),
                    String::from("Q1"),
                    Box::new(|_r| true),
                    Box::new(|r| {
                        let mut s = r.clone();
                        s.t2 = r.x;
                        s
                    }),
                )],
            ),
            (
                String::from("Q1"),
                vec![Trans::new(
                    String::from("inc"),
                    String::from("Q2"),
                    Box::new(|_r| true),
                    Box::new(|r| {
                        let mut s = r.clone();
                        s.t2 = r.t2 + 1;
                        s
                    }),
                )],
            ),
            (
                String::from("Q2"),
                vec![Trans::new(
                    String::from("write"),
                    String::from("Q3"),
                    Box::new(|_r| true),
                    Box::new(|r| {
                        let mut s = r.clone();
                        s.x = r.t2;
                        s
                    }),
                )],
            ),
            (String::from("Q3"), vec![]),
        ],
    };
    process_p.viz_process("p1");
    process_q.viz_process("q")
}
