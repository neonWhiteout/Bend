use crate::{
  diagnostics::Info,
  term::{Ctx, Name, Pattern, Term},
  CORE_BUILTINS,
};
use std::{
  collections::{HashMap, HashSet},
  fmt::Display,
};

#[derive(Debug, Clone)]
pub struct ReferencedMainErr;

impl Display for ReferencedMainErr {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Main definition can't be referenced inside the program.")
  }
}

impl Ctx<'_> {
  /// Decides if names inside a term belong to a Var or to a Ref.
  /// Precondition: Refs are encoded as vars, Constructors are resolved.
  /// Postcondition: Refs are encoded as refs, with the correct def id.
  pub fn resolve_refs(&mut self) -> Result<(), Info> {
    self.info.start_pass();

    let def_names = self.book.defs.keys().cloned().collect::<HashSet<_>>();
    for (def_name, def) in &mut self.book.defs {
      for rule in def.rules.iter_mut() {
        let mut scope = HashMap::new();

        for name in rule.pats.iter().flat_map(Pattern::named_binds) {
          push_scope(Some(name), &mut scope);
        }

        let res = rule.body.resolve_refs(&def_names, self.book.entrypoint.as_ref(), &mut scope);
        self.info.take_err(res, Some(def_name));
      }
    }

    self.info.fatal(())
  }
}

impl Term {
  pub fn resolve_refs<'a>(
    &'a mut self,
    def_names: &HashSet<Name>,
    main: Option<&Name>,
    scope: &mut HashMap<&'a Name, usize>,
  ) -> Result<(), ReferencedMainErr> {
    Term::recursive_call(move || {
      match self {
        // If variable not defined, we check if it's a ref and swap if it is.
        Term::Var { nam } => {
          if is_var_in_scope(nam, scope) {
            if let Some(main) = main {
              if nam == main {
                return Err(ReferencedMainErr);
              }
            }

            if def_names.contains(nam) || CORE_BUILTINS.contains(&nam.0.as_ref()) {
              *self = Term::r#ref(nam);
            }
          }
        }
        _ => {
          for (child, binds) in self.children_mut_with_binds() {
            for bind in binds.iter() {
              push_scope(bind.as_ref(), scope);
            }
            child.resolve_refs(def_names, main, scope)?;
            for bind in binds.iter() {
              pop_scope(bind.as_ref(), scope);
            }
          }
        }
      }
      Ok(())
    })
  }
}

fn push_scope<'a>(name: Option<&'a Name>, scope: &mut HashMap<&'a Name, usize>) {
  if let Some(name) = name {
    let var_scope = scope.entry(name).or_default();
    *var_scope += 1;
  }
}

fn pop_scope<'a>(name: Option<&'a Name>, scope: &mut HashMap<&'a Name, usize>) {
  if let Some(name) = name {
    let var_scope = scope.entry(name).or_default();
    *var_scope -= 1;
  }
}

fn is_var_in_scope<'a>(name: &'a Name, scope: &HashMap<&'a Name, usize>) -> bool {
  match scope.get(name) {
    Some(entry) => *entry == 0,
    None => true,
  }
}
