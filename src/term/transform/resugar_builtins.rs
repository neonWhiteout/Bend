use crate::term::{Term, LCONS, LNIL, SCONS, SNIL};

impl Term {
  pub fn resugar_builtins(&mut self) {
    self.resugar_strings();
    self.resugar_lists();
  }

  /// Rebuilds the String syntax sugar, converting `(Cons 97 Nil)` into `"a"`.
  pub fn resugar_strings(&mut self) {
    match self {
      // (String.cons Num tail)
      Term::App {
        fun: box Term::App { fun: box Term::Ref { nam: ctr }, arg: box head, .. },
        arg: box tail,
        ..
      } => {
        head.resugar_strings();
        tail.resugar_strings();

        if ctr == SCONS
          && let Term::Num { val } = head
          && let Term::Str { val: tail } = tail
        {
          // If well formed string, add the next character to the string we're building
          let head = unsafe { char::from_u32_unchecked(*val as u32) }.to_string();
          let str = head + &tail;
          *self = Term::str(&str);
        } else {
          // Otherwise rebuild the constructor with the new tail

          // Create `(Cons head Nil)` instead of `(Cons head "")`
          if matches!(&tail, Term::Str { val } if val.is_empty()) {
            *tail = Term::r#ref(SNIL);
          }

          let head = std::mem::take(head);
          let tail = std::mem::take(tail);

          *self = Term::call(Term::Ref { nam: ctr.clone() }, [head, tail]);
        }
      }
      // (String.nil)
      Term::Ref { nam: def_name } if def_name == SNIL => *self = Term::str(""),

      _ => {
        for child in self.children_mut() {
          child.resugar_strings();
        }
      }
    }
  }

  /// Rebuilds the List syntax sugar, converting `(Cons head Nil)` into `[head]`.
  pub fn resugar_lists(&mut self) {
    match self {
      // (List.cons el tail)
      Term::App {
        fun: box Term::App { fun: box Term::Ref { nam: ctr }, arg: box head, .. },
        arg: box tail,
        ..
      } => {
        head.resugar_lists();
        tail.resugar_lists();
        let head = std::mem::take(head);

        if ctr == LCONS
          && let Term::Lst { els: tail } = tail
        {
          // If well formed list, cons the next element to the list being formed
          let mut els = vec![head];
          els.extend(std::mem::take(tail));
          *self = Term::Lst { els };
        } else {
          let tail = std::mem::take(tail);
          *self = Term::call(Term::Ref { nam: ctr.clone() }, [head, tail]);
        }
      }
      // (List.nil)
      Term::Ref { nam: def_name } if def_name == LNIL => *self = Term::Lst { els: vec![] },

      _ => {
        for child in self.children_mut() {
          child.resugar_lists();
        }
      }
    }
  }
}
