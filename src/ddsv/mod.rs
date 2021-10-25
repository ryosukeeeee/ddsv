use std::rc::Rc;

pub mod data;

// ラベル（ex. read）
pub type Label = String;
// ロケーション（ex. P1）
pub type Location = String;
// 状態は共有変数とロケーションの配列（各スレッドの状態）から成る
pub type State<T> = (T, Vec<Location>);
pub type Path<T> = Vec<(Label, State<T>)>;

// 状態遷移（プロセス）はロケーション（ex. P1）と遷移リストをタプル要素としてもつリスト
pub type Process<T> = Vec<(Location, Vec<Trans<T>>)>;

// ある状態から遷移の可能性を表す
#[derive(Clone)]
pub struct Trans<T: Clone> {
    pub label: Label,
    pub location: Location,
    pub guard: Rc<dyn Fn(&T) -> bool>,
    pub action: Rc<dyn Fn(&T) -> T>,
}

macro_rules! trans {
    ($t:ty, [$($tuple: expr),*]) => {{
        let mut t: Vec<$crate::ddsv::Trans<$t>> = Vec::new();
        $(
            let trans = $crate::ddsv::Trans {
                label: String::from($tuple.0),
                location: String::from($tuple.1),
                guard: Rc::new($tuple.2),
                action: Rc::new($tuple.3),
            };
            t.push(trans);
        )*
        t
    }};
}

#[macro_export]
macro_rules! process {
    ($t:ty,[
        $(
            (
                $loc:expr, [
                    $($tuple: expr),*
                ]
            )
        ),*
    ]
    ) => {
        {
            let mut p: $crate::ddsv::Process<$t> = Vec::new();
            $(
                let location = String::from($loc);
                let mut t: Vec<$crate::ddsv::Trans<$t>> = Vec::new();
                $(
                    let trans = $crate::ddsv::Trans {
                        label: String::from($tuple.0),
                        location: String::from($tuple.1),
                        guard: Rc::new($tuple.2),
                        action: Rc::new($tuple.3)
                    };
                    t.push(trans);
                )*
                p.push((location, t));
            )*
            p
        }
    };
}

#[cfg(test)]
mod tests {
    use super::data::*;
    use crate::ddsv::Trans;
    use env_logger;
    use std::env;
    use std::rc::Rc;
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

    #[test]
    fn trans_print_test() {
        let mut t: Vec<Trans<SharedVars>> = Vec::new();

        let trans = Trans {
            label: String::from("read"),
            location: String::from("P1"),
            guard: Rc::new(|_: &SharedVars| true),
            action: Rc::new(|r: &SharedVars| r.clone()),
        };
        t.push(trans);
        assert_eq!(
            format!("{:?}", t),
            "[Trans { label: \"read\", location: \"P1\" }]"
        );
    }

    #[test]
    fn trans_macro_test() {
        let t = trans![
            SharedVars,
            [
                (
                    "read",
                    "P1",
                    |_: &SharedVars| true,
                    |r: &SharedVars| r.clone()
                ),
                (
                    "write",
                    "P2",
                    |_: &SharedVars| true,
                    |r: &SharedVars| r.clone()
                )
            ]
        ];

        assert_eq!(
            format!("{:?}", t),
            "[Trans { label: \"read\", location: \"P1\" }, Trans { label: \"write\", location: \"P2\" }]"
        );
    }

    #[test]
    fn trans_macro_process_test() {
        let t = process![
            SharedVars,
            [
                (
                    "Q0",
                    [(
                        "read",
                        "P1",
                        |_: &SharedVars| true,
                        |r: &SharedVars| r.clone()
                    )]
                ),
                (
                    "Q1",
                    [(
                        "write",
                        "P2",
                        |_: &SharedVars| true,
                        |r: &SharedVars| r.clone()
                    )]
                )
            ]
        ];

        assert_eq!(
            format!("{:?}", t),
            "[(\"Q0\", [Trans { label: \"read\", location: \"P1\" }]), (\"Q1\", [Trans { label: \"write\", location: \"P2\" }])]"
        );
    }

    #[test]
    fn process_test() {
        let r0 = SharedVars::new();

        let process_p = process![
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
        let process_q = process![
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
        let r1 = r0.clone();
        let v = make_initial_state(r0, &[process_p, process_q]);
        assert_eq!(v.0, r1);
        assert_eq!(v.1[0], "P0");
        assert_eq!(v.1[1], "Q0");
    }

    #[test]
    fn calc_transitions_test() {
        let mut r0 = SharedVars::new();
        let trans = trans![
            SharedVars,
            [(
                "write",
                "Q3",
                |_: &SharedVars| true,
                |r: &SharedVars| {
                    let mut s = r.clone();
                    s.t1 = r.t1 + 1;
                    s
                }
            )]
        ];

        let next = calc_transitions(&r0, &trans);
        assert_eq!(next.len(), 1);
        assert_eq!(next[0].0, "write");
        assert_eq!(next[0].1, "Q3");
        r0.t1 = 1;
        assert_eq!(next[0].2, r0);
    }

    #[test]
    fn collect_trans_test() {
        init();
        let process_p = process![
            SharedVars,
            [
                (
                    "P0",
                    [(
                        "read_p",
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
        let process_q = process![
            SharedVars,
            [
                (
                    "Q0",
                    [(
                        "read_q",
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
        let calcs = collect_trans(
            &SharedVars::new(),
            &[String::from("P0"), String::from("Q0")],
            &[process_p, process_q],
        );
        assert_eq!(calcs.len(), 2);
        assert_eq!(calcs[0].0, "read_p");
        assert_eq!((calcs[0].1).0, SharedVars::new());
        assert_eq!((calcs[0].1).1, ["P1", "Q0"]);
        assert_eq!(calcs[1].0, "read_q");
        assert_eq!((calcs[1].1).0, SharedVars::new());
        assert_eq!((calcs[1].1).1, ["P0", "Q1"]);
    }
}
