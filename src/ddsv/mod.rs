pub mod data;
// pub mod trans;

#[cfg(test)]
mod tests {
    use super::data::*;

    #[test]
    fn trans_test() {
        let t = Trans::new(
            String::from("read"),
            String::from("P1"),
            Box::new(|_r| true),
            Box::new(|r| r),
        );
        assert_eq!(t.label, String::from("read"));
        assert_eq!(t.location, String::from("P1"));
        assert_eq!((t.guard)(SharedVars::new()), true);
        assert_eq!((t.action)(SharedVars::new()), SharedVars::new());
    }
    #[test]
    fn trans_print_test() {
        let t = Trans::new(
            String::from("read"),
            String::from("P1"),
            Box::new(|_r| true),
            Box::new(|r| r),
        );
        assert_eq!(
            format!("{:?}", t),
            "Trans { label: \"read\", location: \"P1\" }"
        );
    }

    #[test]
    fn process_test() {
        let r0 = SharedVars { x: 0, t1: 0, t2: 0 };
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
                            s.x = r.t2 + 1;
                            s
                        }),
                    )],
                ),
                (String::from("Q3"), vec![]),
            ],
        };
        let v = make_initial_state(&r0, &vec![process_p, process_q]);
        assert_eq!(v.0, r0.clone());
        assert_eq!(v.1[0], "P0");
        assert_eq!(v.1[1], "Q0");
    }

    #[test]
    fn calc_transitions_test() {
        let r0 = SharedVars { x: 0, t1: 0, t2: 0 };
        let next = calc_transitions(
            vec![],
            &r0,
            &[String::from("P1")],
            &[String::from("Q1")],
            &[Trans::new(
                String::from("write"),
                String::from("Q3"),
                Box::new(|_r| true),
                Box::new(|r| {
                    let mut s = r.clone();
                    s.x = r.t2 + 1;
                    s
                }),
            )],
        );
        assert_eq!(next.len(), 1);
        assert_eq!(next[0].0, "write");
        let mut r1 = r0.clone();
        r1.x = 1;
        assert_eq!((next[0].1).0, r1);
        assert_eq!((next[0].1).1, vec!["P1", "Q3", "Q1"]);
        println!("next[0]: {:?}", next[0]);
    }

    #[test]
    fn collect_trans_test() {
        let calcs = collect_trans(
            vec![],
            &SharedVars { x: 0, t1: 0, t2: 0 },
            &[],
            &[String::from("P0"), String::from("Q0")],
            &vec![
                Process {
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
                },
                Process {
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
                                    s.x = r.t2 + 1;
                                    s
                                }),
                            )],
                        ),
                        (String::from("Q3"), vec![]),
                    ],
                },
            ],
        );
        println!("collect_trans: {:?}", calcs);
    }
}
