//! [β-reduction](https://en.wikipedia.org/wiki/Beta_normal_form) for lambda `Term`s

use term::*;
use term::Term::*;
use self::Order::*;
use std::fmt;

/// Set to `true` to see all the steps of β-reductions. The default is `false`.
pub const SHOW_REDUCTIONS: bool = false;

/// The [evaluation order](http://www.cs.cornell.edu/courses/cs6110/2014sp/Handouts/Sestoft.pdf) of
/// β-reductions. The default is `NOR` (normal order).
///
/// They don't always yield the same result:
///
/// - The `NOR`, `HNO`, `APP` and `HAP` orders reduce expressions to their normal forms
/// - The `APP` order will fail to fully reduce expressions containing functions without a normal
/// form, e.g. the `Y` combinator (they will expand forever)
/// - The `CBN` order reduces to weak head normal form
/// - The `CBV` order reduces to weak normal form
/// - The `HSP` order reduces to head normal form
#[derive(Debug, PartialEq)]
pub enum Order {
    /// Normal - leftmost outermost
    NOR,
    /// Call-by-name - leftmost outermost, but not inside abstractions
    CBN,
    /// Head spine - leftmost outermost, but abstractions reduced only in head position
    HSP,
    /// Hybrid normal - a hybrid between `HSP` and `NOR`
    HNO,
    /// Applicative - leftmost innermost
    APP,
    /// Call-by-value - leftmost innermost, but not inside abstractions
    CBV,
    /// Hybrid applicative - a hybrid between `CBV` and `APP`
    HAP
}

/// Applies two `Term`s with substitution and variable update, consuming the first one in the
/// process. It produces an `Error` if the first `Term` is not an `Abs`traction.
///
/// # Example
/// ```
/// use lambda_calculus::reduction::apply;
/// use lambda_calculus::parser::parse;
///
/// let lhs    = parse(&"λλ42(λ13)").unwrap();
/// let rhs    = parse(&"λ51").unwrap();
/// let result = parse(&"λ3(λ61)(λ1(λ71))").unwrap();
///
/// assert_eq!(apply(lhs, &rhs), Ok(result));
/// ```
pub fn apply(mut lhs: Term, rhs: &Term) -> Result<Term, Error> {
    _apply(&mut lhs, rhs, 0);

    lhs.unabs()
}

fn _apply(lhs: &mut Term, rhs: &Term, depth: usize) {
    match *lhs {
        Var(i) => if i == depth {
            *lhs = rhs.clone(); // substitute a top-level variable from lhs with rhs
            update_free_variables(lhs, depth - 1, 0); // update indices of free variables from rhs
        } else if i > depth {
            *lhs = Var(i - 1) // decrement a free variable's index
        },
        Abs(_) => {
            _apply(lhs.unabs_ref_mut().unwrap(), rhs, depth + 1)
        },
        App(_, _) => {
            _apply(lhs.lhs_ref_mut().unwrap(), rhs, depth);
            _apply(lhs.rhs_ref_mut().unwrap(), rhs, depth)
        }
    }
}

fn update_free_variables(term: &mut Term, added_depth: usize, own_depth: usize) {
    match *term {
        Var(i) => if i > own_depth {
            *term = Var(i + added_depth)
        },
        Abs(_) => {
            update_free_variables(term.unabs_ref_mut().unwrap(), added_depth, own_depth + 1)
        },
        App(_, _) => {
            update_free_variables(term.lhs_ref_mut().unwrap(), added_depth, own_depth);
            update_free_variables(term.rhs_ref_mut().unwrap(), added_depth, own_depth)
        }
    }
}

/// Performs β-reduction on a `Term` with the specified evaluation `Order` and an optional limit on
/// number of reductions (`0` means no limit).
///
/// # Example
///
/// ```
/// use lambda_calculus::arithmetic::pred;
/// use lambda_calculus::reduction::beta;
/// use lambda_calculus::reduction::Order::NOR;
///
/// let pred_one = pred().app(1.into());
///
/// assert_eq!(beta(pred_one, NOR, 0), 0.into());
/// ```
pub fn beta(mut term: Term, order: Order, limit: usize) -> Term {
    term.beta(order, limit);
    term
}

/// Prints the number of reductions required for `term` to reach the final form with all the
/// available reduction strategies, optionally excluding the ones from the given list. Such
/// exclusions might be necessary, especially if the expression contains the fixed-point combinator.
///
/// # Example
///
/// ```
/// use lambda_calculus::arithmetic::fac;
/// use lambda_calculus::reduction::benchmark;
/// use lambda_calculus::reduction::Order::*;
///
/// benchmark(&fac().app(3.into()), &[CBN, CBV, HSP]);
///
/// // stdout:
///
/// // normal:             46
/// // applicative:        39
/// // hybrid normal:      46
/// // hybrid applicative: 39
/// ```
pub fn benchmark(term: &Term, exclude: &[Order]) {
    let mut count = 0;
    if !exclude.contains(&CBN) {
        term.clone().beta_cbn(0, 0, &mut count);
        println!("{}:       {}", CBN, count);
        count = 0;
    }
    if !exclude.contains(&NOR) {
        term.clone().beta_nor(0, 0, &mut count);
        println!("{}:             {}", NOR, count);
        count = 0;
    }
    if !exclude.contains(&CBV) {
        term.clone().beta_cbv(0, 0, &mut count);
        println!("{}:      {}", CBV, count);
        count = 0;
    }
    if !exclude.contains(&APP) {
        term.clone().beta_app(0, 0, &mut count);
        println!("{}:        {}", APP, count);
        count = 0;
    }
    if !exclude.contains(&HSP) {
        term.clone().beta_hsp(0, 0, &mut count);
        println!("{}:         {}", HSP, count);
        count = 0;
    }
    if !exclude.contains(&HNO) {
        term.clone().beta_hno(0, 0, &mut count);
        println!("{}:      {}", HNO, count);
        count = 0;
    }
    if !exclude.contains(&HAP) {
        term.clone().beta_hap(0, 0, &mut count);
        println!("{}: {}", HAP, count);
    }
}

impl Term {
    /// Applies `self` to another `Term` and performs substitution, consuming `self` in the process.
    /// It produces an `Error` if `self` is not an `Abs`traction.
    ///
    /// # Example
    /// ```
    /// use lambda_calculus::parser::parse;
    ///
    /// let lhs    = parse(&"λλ42(λ13)").unwrap();
    /// let rhs    = parse(&"λ51").unwrap();
    /// let result = parse(&"λ3(λ61)(λ1(λ71))").unwrap();
    ///
    /// assert_eq!(lhs.apply(&rhs), Ok(result));
    /// ```
    pub fn apply(self, rhs: &Term) -> Result<Term, Error> {
        apply(self, rhs)
    }

    /// Reduces an `App`lication by substitution and variable update. It produces an `Error` if
    /// `self` is not an `App`lication.
    ///
    /// # Example
    /// ```
    /// use lambda_calculus::term::{app, abs};
    /// use lambda_calculus::term::Term::Var;
    /// use lambda_calculus::arithmetic::zero;
    /// use lambda_calculus::combinators::i;
    ///
    /// assert_eq!(app(i(), zero()).eval(), Ok(zero()));
    /// ```
    pub fn eval(self) -> Result<Term, Error> {
        let (lhs, rhs) = try!(self.unapp());

        apply(lhs, &rhs)
    }

    fn eval_with_info(&mut self, depth: u32, count: &usize) {
        if SHOW_REDUCTIONS {
            print!("\n{}. {}\n=>", count + 1, show_precedence(self, 0, depth));
            let mut indent_len = ((*count + 1) as f32).log10().trunc() as usize + 3;
            if DISPLAY_CLASSIC { indent_len += 3 }
            let lhs = self.lhs_ref().unwrap();
            let rhs = self.rhs_ref().unwrap();
            if lhs.unabs_ref().unwrap().unvar_ref().is_ok() {
                if DISPLAY_CLASSIC { indent_len += 3 } else { indent_len += 2 }
                if rhs.unapp_ref().is_ok() { indent_len += 1 }
            }

            for _ in 0..indent_len { print!(" ") };
        };

        let copy = self.clone();
        *self = copy.eval().unwrap();

        if SHOW_REDUCTIONS {
            if self.unvar_ref().is_ok() {
                println!("{}", show_precedence(self, 0, depth - 1))
            } else {
                println!("{}", show_precedence(self, 0, depth))
            }
        };
    }

    fn is_reducible(&self, limit: usize, count: &usize) -> bool {
        self.lhs_ref().unwrap().unabs_ref().is_ok() && (limit == 0 || *count < limit )
    }

    /// Performs β-reduction on a `Term` with the specified evaluation `Order` and an optional
    /// limit on number of reductions (`0` means no limit).
    ///
    /// # Example
    ///
    /// ```
    /// use lambda_calculus::arithmetic::pred;
    /// use lambda_calculus::reduction::Order::NOR;
    ///
    /// let mut pred_one = pred().app(1.into());
    /// pred_one.beta(NOR, 0);
    ///
    /// assert_eq!(pred_one, 0.into());
    /// ```
    pub fn beta(&mut self, order: Order, limit: usize) {
        if SHOW_REDUCTIONS {
            println!("reducing {} [{} order{}]:", self, order,
                if limit != 0 {
                    format!(", limit of {} reduction{}", limit, if limit == 1 { "" } else { "s" })
                } else {
                    "".into()
                }
            );
        };

        let mut count = 0;

        match order {
            CBN => self.beta_cbn(0, limit, &mut count),
            NOR => self.beta_nor(0, limit, &mut count),
            CBV => self.beta_cbv(0, limit, &mut count),
            APP => self.beta_app(0, limit, &mut count),
            HSP => self.beta_hsp(0, limit, &mut count),
            HNO => self.beta_hno(0, limit, &mut count),
            HAP => self.beta_hap(0, limit, &mut count)
        }
        if SHOW_REDUCTIONS {
            println!("\nresult after {} reduction{}: {}\n", count,
                if limit == 1 { "" } else { "s" }, self);
        };
    }

    fn beta_cbn(&mut self, depth: u32, limit: usize, count: &mut usize) {
        if limit != 0 && *count == limit { return }

        match *self {
            App(_, _) => {
                self.lhs_ref_mut().unwrap().beta_cbn(depth, limit, count);

                if self.is_reducible(limit, count) {
                    self.eval_with_info(depth, count);
                    *count += 1;
                    self.beta_cbn(depth, limit, count);
                }
            },
            _ => ()
        }
    }

    fn beta_nor(&mut self, depth: u32, limit: usize, count: &mut usize) {
        if limit != 0 && *count == limit { return }

        match *self {
            Var(_) => (),
            Abs(_) => self.unabs_ref_mut().unwrap().beta_nor(depth + 1, limit, count),
            App(_, _) => {
                self.lhs_ref_mut().unwrap().beta_cbn(depth, limit, count);

                if self.is_reducible(limit, count) {
                    self.eval_with_info(depth, count);
                    *count += 1;
                    self.beta_nor(depth, limit, count);
                } else {
                    self.lhs_ref_mut().unwrap().beta_nor(depth, limit, count);
                    self.rhs_ref_mut().unwrap().beta_nor(depth, limit, count);
                }
            }
        }
    }

    fn beta_cbv(&mut self, depth: u32, limit: usize, count: &mut usize) {
        if limit != 0 && *count == limit { return }

        match *self {
            App(_, _) => {
                self.lhs_ref_mut().unwrap().beta_cbv(depth, limit, count);
                self.rhs_ref_mut().unwrap().beta_cbv(depth, limit, count);

                if self.is_reducible(limit, count) {
                    self.eval_with_info(depth, count);
                    *count += 1;
                    self.beta_cbv(depth, limit, count);
                }
            },
            _ => ()
        }
    }

    fn beta_app(&mut self, depth: u32, limit: usize, count: &mut usize) {
        if limit != 0 && *count == limit { return }

        match *self {
            Var(_) => (),
            Abs(_) => self.unabs_ref_mut().unwrap().beta_app(depth + 1, limit, count),
            App(_, _) => {
                self.lhs_ref_mut().unwrap().beta_app(depth, limit, count);
                self.rhs_ref_mut().unwrap().beta_app(depth, limit, count);

                if self.is_reducible(limit, count) {
                    self.eval_with_info(depth, count);
                    *count += 1;
                    self.beta_app(depth, limit, count);
                }
            }
        }
    }

    fn beta_hap(&mut self, depth: u32, limit: usize, count: &mut usize) {
        if limit != 0 && *count == limit { return }

        match *self {
            Var(_) => (),
            Abs(_) => self.unabs_ref_mut().unwrap().beta_hap(depth + 1, limit, count),
            App(_, _) => {
                self.lhs_ref_mut().unwrap().beta_cbv(depth, limit, count);
                self.rhs_ref_mut().unwrap().beta_hap(depth, limit, count);

                if self.is_reducible(limit, count) {
                    self.eval_with_info(depth, count);
                    *count += 1;
                    self.beta_hap(depth, limit, count);
                } else {
                    self.lhs_ref_mut().unwrap().beta_hap(depth, limit, count);
                }
            }
        }
    }

    fn beta_hsp(&mut self, depth: u32, limit: usize, count: &mut usize) {
        if limit != 0 && *count == limit { return }

        match *self {
            Var(_) => (),
            Abs(_) => self.unabs_ref_mut().unwrap().beta_hsp(depth + 1, limit, count),
            App(_, _) => {
                self.lhs_ref_mut().unwrap().beta_cbn(depth, limit, count);

                if self.is_reducible(limit, count) {
                    self.eval_with_info(depth, count);
                    *count += 1;
                    self.beta_hsp(depth, limit, count)
                }
            }
        }
    }

    fn beta_hno(&mut self, depth: u32, limit: usize, count: &mut usize) {
        if limit != 0 && *count == limit { return }

        match *self {
            Var(_) => (),
            Abs(_) => self.unabs_ref_mut().unwrap().beta_hno(depth + 1, limit, count),
            App(_, _) => {
                self.lhs_ref_mut().unwrap().beta_hsp(depth, limit, count);

                if self.is_reducible(limit, count) {
                    self.eval_with_info(depth, count);
                    *count += 1;
                    self.beta_hno(depth, limit, count)
                } else {
                    self.lhs_ref_mut().unwrap().beta_hno(depth, limit, count);
                    self.rhs_ref_mut().unwrap().beta_hno(depth, limit, count);
                }
            }
        }
    }
}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match *self {
            NOR => "normal",
            CBN => "call-by-name",
            HSP => "head spine",
            HNO => "hybrid normal",
            APP => "applicative",
            CBV => "call-by-value",
            HAP => "hybrid applicative"
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use parser::parse;
    use combinators::{i, omm};
    use arithmetic::fac;
    use std::thread;

    #[test]
    fn normal_order() {
        let reduces_instantly = parse(&"(λλ1)((λλλ((32)1))(λλ2))").unwrap();
        assert_eq!(beta(reduces_instantly.clone(), NOR, 0),
                   beta(reduces_instantly,         NOR, 1)
        );

        let should_reduce = parse(&"(λ2)((λ111)(λ111))").unwrap();
        assert_eq!(beta(should_reduce, NOR, 0), Var(1));

        let does_reduce = app(abs(Var(2)), omm());
        assert_eq!(beta(does_reduce, NOR, 0), Var(1));
    }

    #[test]
    fn call_by_name_order() {
        let mut expr = app(abs(app(i(), Var(1))), app(i(), i()));
        expr.beta(CBN, 1);
        assert_eq!(expr, app(i(), app(i(), i())));
        expr.beta(CBN, 1);
        assert_eq!(expr, app(i(), i()));
        expr.beta(CBN, 1);
        assert_eq!(expr, i());
    }

    #[test]
    fn applicative_order() {
        let mut wont_reduce = app(abs(Var(2)), omm());
        wont_reduce.beta(APP, 3);
        assert_eq!(wont_reduce, app(abs(Var(2)), omm()));
    }

    #[test]
    fn call_by_value_order() {
        let mut expr = app(abs(app(i(), Var(1))), app(i(), i()));
        expr.beta(CBV, 1);
        assert_eq!(expr, app(abs(app(i(), Var(1))), i()));
        expr.beta(CBV, 1);
        assert_eq!(expr, app(i(), i()));
        expr.beta(CBV, 1);
        assert_eq!(expr, i());
    }

    #[test]
    #[ignore]
    fn huge_reduction() {
        let builder = thread::Builder::new().name("reductor".into()).stack_size(2048 * 1024 * 1024);

        let handler = builder.spawn(|| {
            assert_eq!(beta(app!(fac(), 10.into()), HAP, 0).value(), Ok(3628800));
        }).unwrap();

        handler.join().unwrap();
    }
}
