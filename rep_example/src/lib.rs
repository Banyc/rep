use rep::*;
extern crate log;

fn is_gt_zero(num: i32) -> bool {
    num > 0
}

#[derive(CheckIndieFields)]
struct Point {
    #[rep(assert_eq = 0)]
    x: i32,
    y: i32,
}

#[derive(CheckIndieFields)]
struct Line {
    // #[rep(assert_default)]
    // #[rep(assert_true)]
    // #[rep(assert_false)]
    // #[rep(assert_eq = 0.0)]
    // #[rep(assert_eq = true)]
    // #[rep(assert_eq = "hello")]
    // #[rep(assert_gt = 0.0)]
    // #[rep(assert_lt = 10.0)]
    // #[rep(assert_ge = 0.0)]
    // #[rep(assert_le = 10.0)]
    // #[rep(check)]
    start: Point,
    #[rep(assert_with = "is_gt_zero")]
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}

impl CheckFields for Line {
    fn check_fields(&self, e: &mut RepErrors) {
        if self.x2 != self.y2 {
            e.add(String::from("self.x2 must equal self.y2"));
        }
    }
}

impl CheckRep for Line {}

impl Line {
    #[ensure_rep]
    pub fn foo(&mut self) {}
}

#[check_rep]
impl Line {
    pub fn bad_new() -> Self {
        Line {
            start: Point { x: 0, y: 0 },
            x1: 0,
            y1: 0,
            x2: 0,
            y2: 0,
        }
    }

    pub fn good_new() -> Self {
        Line {
            start: Point { x: 0, y: 0 },
            x1: 1,
            y1: 0,
            x2: 0,
            y2: 0,
        }
    }

    pub fn set_x1(&mut self, x1: i32) -> i32 {
        let old = self.x1;
        self.x1 = x1;
        old
    }
}

#[ensure_rep]
impl Line {
    pub fn set_x1_2(&mut self, x1: i32) -> i32 {
        let old = self.x1;
        self.x1 = x1;
        old
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(
        expected = "representation invariant violated: RepErrors { errors: [\"is_gt_zero(self.x1) must be true when self.x1 == -20\", \"self.x2 must equal self.y2\"] }"
    )]
    fn test_ensure_rep() {
        // env_logger::init();

        let mut line = Line {
            start: Point { x: 50, y: 50 },
            x1: -20,
            y1: 0,
            x2: 5,
            y2: 10,
        };

        line.foo();
    }

    #[test]
    fn test_ensure_rep_ok() {
        let mut line = Line {
            start: Point { x: 0, y: 0 },
            x1: 0,
            y1: 0,
            x2: 0,
            y2: 0,
        };

        line.set_x1_2(1);
    }

    #[test]
    #[should_panic(
        expected = "representation invariant violated: RepErrors { errors: [\"is_gt_zero(self.x1) must be true when self.x1 == 0\"] }"
    )]
    fn test_new_self() {
        let _line = Line::bad_new();
    }

    #[test]
    #[should_panic(
        expected = "representation invariant violated: RepErrors { errors: [\"is_gt_zero(self.x1) must be true when self.x1 == 0\"] }"
    )]
    fn test_mut_self_with_returns() {
        let mut line = Line::good_new();
        line.set_x1(0);
    }
}
