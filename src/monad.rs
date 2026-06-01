//! Monads — computational effects as monads (Maybe for nullable, State for mutation, Reader for config).

use serde::{Deserialize, Serialize};

/// A monad descriptor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Monad {
    pub name: String,
    pub description: String,
    pub kind: MonadKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MonadKind {
    Maybe,
    State,
    Reader,
    Writer,
    Either,
    List,
    IO,
    Custom(String),
}

impl Monad {
    pub fn new(name: impl Into<String>, kind: MonadKind) -> Self {
        Monad {
            name: name.into(),
            description: String::new(),
            kind,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }
}

/// Maybe monad — represents nullable/optional values.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Maybe<T> {
    Just(T),
    Nothing,
}

impl<T: Clone> Maybe<T> {
    pub fn return_(value: T) -> Self {
        Maybe::Just(value)
    }

    pub fn bind<U, F>(&self, f: F) -> Maybe<U>
    where
        F: Fn(&T) -> Maybe<U>,
    {
        match self {
            Maybe::Just(v) => f(v),
            Maybe::Nothing => Maybe::Nothing,
        }
    }

    pub fn is_just(&self) -> bool {
        matches!(self, Maybe::Just(_))
    }

    pub fn is_nothing(&self) -> bool {
        matches!(self, Maybe::Nothing)
    }
}

/// State monad — represents stateful computations (not serializable due to closures).
pub struct State<S, A> {
    pub run: Box<dyn Fn(S) -> (A, S)>,
}

impl<S, A> std::fmt::Debug for State<S, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "State<...>")
    }
}

impl<S: Clone + 'static, A: Clone + 'static> State<S, A> {
    pub fn return_(value: A) -> Self {
        let v = value;
        State {
            run: Box::new(move |s| (v.clone(), s)),
        }
    }

    pub fn bind<U: Clone + 'static>(self, f: impl Fn(A) -> State<S, U> + 'static) -> State<S, U> {
        let run = self.run;
        State {
            run: Box::new(move |s| {
                let (a, s2) = run(s);
                (f(a).run)(s2)
            }),
        }
    }

    pub fn get() -> State<S, S> {
        State {
            run: Box::new(|s| (s.clone(), s)),
        }
    }

    pub fn put(new_state: S) -> State<S, ()> {
        State {
            run: Box::new(move |_| ((), new_state.clone())),
        }
    }

    pub fn eval(&self, s: S) -> A {
        (self.run)(s).0
    }
}

/// Reader monad — represents computations that read from a shared environment.
pub struct Reader<R, A> {
    pub run: Box<dyn Fn(&R) -> A>,
}

impl<R, A> std::fmt::Debug for Reader<R, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Reader<...>")
    }
}

impl<R: Clone + 'static, A: Clone + 'static> Reader<R, A> {
    pub fn return_(value: A) -> Self {
        let v = value;
        Reader {
            run: Box::new(move |_| v.clone()),
        }
    }

    pub fn bind<B: Clone + 'static>(self, f: impl Fn(A) -> Reader<R, B> + 'static) -> Reader<R, B> {
        let run = self.run;
        Reader {
            run: Box::new(move |env| {
                let a = run(env);
                (f(a).run)(env)
            }),
        }
    }

    pub fn ask() -> Reader<R, R> {
        Reader { run: Box::new(|env| env.clone()) }
    }

    pub fn eval(&self, env: &R) -> A {
        (self.run)(env)
    }
}

/// Writer monad — represents computations that produce a log/trace.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Writer<W, A> {
    pub value: A,
    pub log: Vec<W>,
}

impl<W: Clone, A: Clone> Writer<W, A> {
    pub fn return_(value: A) -> Self {
        Writer { value, log: vec![] }
    }

    pub fn bind<U>(self, f: impl Fn(A) -> Writer<W, U>) -> Writer<W, U> {
        let Writer { value, log: log1 } = self;
        let Writer { value: new_val, log: log2 } = f(value);
        let mut combined = log1;
        combined.extend(log2);
        Writer { value: new_val, log: combined }
    }

    pub fn tell(entry: W) -> Writer<W, ()> {
        Writer { value: (), log: vec![entry] }
    }
}

/// Monad transformer stack — for composing multiple monadic effects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonadStack {
    pub name: String,
    pub layers: Vec<MonadKind>,
}

impl MonadStack {
    pub fn new(name: impl Into<String>) -> Self {
        MonadStack {
            name: name.into(),
            layers: Vec::new(),
        }
    }

    pub fn push(mut self, m: MonadKind) -> Self {
        self.layers.push(m);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maybe_return() {
        let m = Maybe::return_(42);
        assert_eq!(m, Maybe::Just(42));
    }

    #[test]
    fn test_maybe_bind_just() {
        let m = Maybe::Just(5);
        let result = m.bind(|x| Maybe::Just(x * 2));
        assert_eq!(result, Maybe::Just(10));
    }

    #[test]
    fn test_maybe_bind_nothing() {
        let m: Maybe<i32> = Maybe::Nothing;
        let result = m.bind(|x| Maybe::Just(x * 2));
        assert_eq!(result, Maybe::Nothing);
    }

    #[test]
    fn test_maybe_left_identity() {
        let a = 3;
        let f = |x: &i32| Maybe::Just(*x + 1);
        let left = Maybe::return_(a).bind(f);
        let right = f(&a);
        assert_eq!(left, right);
    }

    #[test]
    fn test_maybe_right_identity() {
        let m = Maybe::Just(7);
        let result = m.bind(|x| Maybe::return_(*x));
        assert_eq!(result, Maybe::Just(7));
    }

    #[test]
    fn test_maybe_associativity() {
        let m = Maybe::Just(3);
        let f = |x: &i32| Maybe::Just(*x + 1);
        let g = |x: &i32| Maybe::Just(*x * 2);
        let left = m.bind(&f).bind(&g);
        let right = m.bind(move |x| f(x).bind(&g));
        assert_eq!(left, right);
    }

    #[test]
    fn test_maybe_is_checks() {
        assert!(Maybe::Just(1).is_just());
        assert!(!Maybe::Just(1).is_nothing());
        assert!(Maybe::<i32>::Nothing.is_nothing());
        assert!(!Maybe::<i32>::Nothing.is_just());
    }

    #[test]
    fn test_writer_return() {
        let w: Writer<&str, i32> = Writer::return_(42);
        assert_eq!(w.value, 42);
        assert!(w.log.is_empty());
    }

    #[test]
    fn test_writer_bind() {
        let w: Writer<&str, i32> = Writer::return_(5);
        let result = w.bind(|x| Writer {
            value: x * 2,
            log: vec!["doubled"],
        });
        assert_eq!(result.value, 10);
        assert_eq!(result.log, vec!["doubled"]);
    }

    #[test]
    fn test_writer_tell() {
        let w = Writer::<&str, ()>::tell("event");
        assert_eq!(w.log, vec!["event"]);
    }

    #[test]
    fn test_writer_monad_left_identity() {
        let f = |x: i32| Writer::<&str, _> { value: x + 1, log: vec![] };
        let left = Writer::<&str, _>::return_(3).bind(&f);
        let right = f(3);
        assert_eq!(left.value, right.value);
    }

    #[test]
    fn test_writer_right_identity() {
        let m = Writer::<&str, _>::return_(3);
        let result = m.bind(|x| Writer::<&str, _>::return_(x));
        assert_eq!(result.value, 3);
    }

    #[test]
    fn test_state_return_and_eval() {
        let s: State<i32, i32> = State::return_(42);
        assert_eq!(s.eval(0), 42);
    }

    #[test]
    fn test_state_get() {
        let s = State::<i32, i32>::get();
        assert_eq!(s.eval(99), 99);
    }

    #[test]
    fn test_state_put() {
        let s = State::<i32, ()>::put(7);
        let (_, new_state) = (s.run)(0);
        assert_eq!(new_state, 7);
    }

    #[test]
    fn test_reader_return_and_eval() {
        let r: Reader<i32, i32> = Reader::return_(42);
        assert_eq!(r.eval(&0), 42);
    }

    #[test]
    fn test_reader_ask() {
        let r = Reader::<i32, i32>::ask();
        assert_eq!(r.eval(&99), 99);
    }

    #[test]
    fn test_monad_creation() {
        let m = Monad::new("StateMonad", MonadKind::State)
            .with_description("Stateful computations");
        assert_eq!(m.name, "StateMonad");
        assert_eq!(m.kind, MonadKind::State);
    }

    #[test]
    fn test_monad_stack() {
        let stack = MonadStack::new("AppState")
            .push(MonadKind::Reader)
            .push(MonadKind::State)
            .push(MonadKind::IO);
        assert_eq!(stack.layers.len(), 3);
    }
}
