/*
use super::*;
use dusk_cdf::*;
use dusk_zkp_debugger_utils::*;

impl<S> App<S> {
    fn override_constraint(&mut self, constraint: Constraint) {
        self.constraint.replace(constraint);
    }

    fn last_constraint(&self) -> usize {
        self.last_constraint
    }
}

impl<S> App<S>
where
    S: io::Read + io::Seek,
{
    fn default_with_cdf(source: S) -> Self {
        CircuitDescription::from_reader(source)
            .and_then(|cdf| {
                let mut app = Self::default();
                app.set_cdf(cdf)?;

                Ok(app)
            })
            .expect("failed to generate app")
    }
}

#[test]
fn breakpoint_and_delete_works() {
    let mut app: App<()> = App::default();

    let file = String::from("gadgets.rs");
    let line = 4837;
    let col = 38;

    app.add_breakpoint(file.clone(), None);

    let id = Default::default();
    let poly = Default::default();
    let source = Source::new(line, col, file.clone().into());

    let constraint = Constraint::new(id, poly, source);

    assert!(!app.is_breakpoint());
    app.override_constraint(constraint.clone());
    assert!(app.is_breakpoint());

    let mut app: App<()> = App::default();

    let id = app.add_breakpoint(file.clone(), Some(line));
    assert!(!app.is_breakpoint());
    app.override_constraint(constraint.clone());
    assert!(app.is_breakpoint());
    app.delete_breakpoint(id);
    assert!(!app.is_breakpoint());
}

#[test]
fn next_afore_and_goto_works() {
    let w_len = 10;
    let c_len = 20;
    let preamble = *Preamble::new()
        .with_witnesses(w_len)
        .with_constraints(c_len);

    let cursor = CDFGenerator::new(0x3489, preamble).gen_cursor();
    let mut app = App::default_with_cdf(cursor);

    assert_eq!(0, app.last_constraint());
    app.next().expect("failed to jump");
    assert_eq!(1, app.last_constraint());
    app.afore().expect("failed to jump");
    assert_eq!(0, app.last_constraint());
    app.goto(10).expect("failed to jump");
    assert_eq!(10, app.last_constraint());
}

#[test]
fn continue_and_turn_works() {
    let w_len = 10;
    let c_len = 20;
    let preamble = *Preamble::new()
        .with_witnesses(w_len)
        .with_constraints(c_len);

    // gen a circuit with all evaluations to true, except the 2nd & 11th
    let mut i = 0;
    let cursor = CDFGenerator::new(0x3489, preamble).gen_cursor_with_callback(
        |w| w,
        move |c| {
            let id = c.id();
            let (qm, ql, qr, qd, qc, qo, pi, qarith, qlogic, qvariable_add, a, b, d, o, _) =
                c.polynomial().internals();
            let poly = Polynomial::new(
                *qm,
                *ql,
                *qr,
                *qd,
                *qc,
                *qo,
                *pi,
                *qarith,
                *qlogic,
                *qvariable_add,
                a.clone(),
                b.clone(),
                d.clone(),
                o.clone(),
                i != 1 && i != 10,
            );
            let source = c.source().clone();

            i += 1;

            Constraint::new(id, poly, source)
        },
    );

    let mut app = App::default_with_cdf(cursor);

    assert_eq!(0, app.last_constraint());
    app.cont().expect("failed to cont");
    assert_eq!(1, app.last_constraint());
    app.cont().expect("failed to cont");
    assert_eq!(10, app.last_constraint());
    app.cont().expect("failed to cont");
    assert_eq!(c_len - 1, app.last_constraint());

    app.turn().expect("failed to turn");
    assert_eq!(10, app.last_constraint());
    app.turn().expect("failed to turn");
    assert_eq!(1, app.last_constraint());
    app.turn().expect("failed to turn");
    assert_eq!(0, app.last_constraint());
}

#[test]
fn load_wont_panic() {
    App::load().ok();
}
*/
