pub mod data;

type Guard<T> = fn(&T) -> bool;
type Action<T> = fn(&T) -> T;
type Label = String;
type Location = String;
type State<T> = (T, Vec<Location>);
type Path<T> = Vec<(Label, State<T>)>;

#[cfg(test)]
mod tests {
    use super::data::*;
    use env_logger;
    use std::env;
    #[derive(Clone, Eq, Hash)]
    struct SharedVars {
        x: i32,
        t1: i32,
        t2: i32,
    }

    impl PartialEq for SharedVars {
        fn eq(&self, other: &Self) -> bool {
            self.x == other.x && self.t1 == other.t1 && self.t2 == other.t2
        }
    }

    impl SharedVars {
        fn new() -> SharedVars {
            SharedVars { x: 0, t1: 0, t2: 0 }
        }
    }

    impl std::fmt::Debug for SharedVars {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
            f.write_fmt(format_args!("x={} t1={} t2={}", self.x, self.t1, self.t2))
        }
    }
    fn init() {
        env::set_var("RUST_LOG", "info");
        env_logger::init();
    }

    fn always_true(_r: &SharedVars) -> bool {
        true
    }
    fn return_copied(r: &SharedVars) -> SharedVars {
        r.clone()
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
    #[test]
    fn trans_test() {
        let t = Trans::new("read", "P1", always_true, return_copied);
        assert_eq!(t.label, String::from("read"));
        assert_eq!(t.location, String::from("P1"));
        assert_eq!((t.guard)(&SharedVars::new()), true);
        assert_eq!((t.action)(&SharedVars::new()), SharedVars::new());
    }
    #[test]
    fn trans_print_test() {
        let t = Trans::new("read", "P1", always_true, return_copied);
        assert_eq!(
            format!("{:?}", t),
            "Trans { label: \"read\", location: \"P1\" }"
        );
    }

    #[test]
    fn process_test() {
        let r0 = SharedVars::new();
        let process_p = Process::new(vec![
            (
                "P0",
                vec![Trans::new("read", "P1", always_true, move_x_to_t1)],
            ),
            (
                "P1",
                vec![Trans::new("inc", "P2", always_true, increment_t1)],
            ),
            (
                "P2",
                vec![Trans::new("write", "P3", always_true, move_t1_to_x)],
            ),
            ("P3", vec![]),
        ]);
        let process_q = Process::new(vec![
            (
                "Q0",
                vec![Trans::new("read", "Q1", always_true, move_x_to_t2)],
            ),
            (
                "Q1",
                vec![Trans::new("inc", "Q2", always_true, increment_t2)],
            ),
            (
                "Q2",
                vec![Trans::new("write", "Q3", always_true, move_t2_to_x)],
            ),
            ("Q3", vec![]),
        ]);
        let v = make_initial_state(&r0, &vec![process_p, process_q]);
        assert_eq!(v.0, r0.clone());
        assert_eq!(v.1[0], "P0");
        assert_eq!(v.1[1], "Q0");
    }

    #[test]
    fn calc_transitions_test() {
        let r0 = SharedVars::new();
        let next = calc_transitions(
            vec![],
            &r0,
            &[String::from("P1")],
            &[String::from("Q1")],
            &[Trans::new("write", "Q3", always_true, increment_t1)],
        );
        assert_eq!(next.len(), 1);
        assert_eq!(next[0].0, "write");
        let mut r1 = r0.clone();
        r1.t1 = 1;
        assert_eq!((next[0].1).0, r1);
        assert_eq!((next[0].1).1, vec!["P1", "Q3", "Q1"]);
        println!("next[0]: {:?}", next[0]);
    }

    #[test]
    fn collect_trans_test() {
        init();
        let calcs = collect_trans(
            vec![],
            &SharedVars::new(),
            &[],
            &[String::from("P0"), String::from("Q0")],
            &vec![
                Process::new(vec![
                    (
                        "P0",
                        vec![Trans::new("read", "P1", always_true, move_x_to_t1)],
                    ),
                    (
                        "P1",
                        vec![Trans::new("inc", "P2", always_true, increment_t1)],
                    ),
                    (
                        "P2",
                        vec![Trans::new("write", "P3", always_true, move_t1_to_x)],
                    ),
                    ("P3", vec![]),
                ]),
                Process::new(vec![
                    (
                        "Q0",
                        vec![Trans::new("read", "Q1", always_true, move_x_to_t2)],
                    ),
                    (
                        "Q1",
                        vec![Trans::new("inc", "Q2", always_true, increment_t2)],
                    ),
                    (
                        "Q2",
                        vec![Trans::new("write", "Q3", always_true, move_t2_to_x)],
                    ),
                    ("Q3", vec![]),
                ]),
            ],
        );
        println!("collect_trans: {:?}", calcs);
    }
}
