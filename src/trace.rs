#[cfg(feature = "trace")]
use {
    crate::STACK,
    std::{
        cell::{Cell, RefCell},
        rc::Rc,
    },
};

#[cfg(feature = "trace")]
thread_local! {
    static BUMP: Cell<usize> = Cell::new(0);
}

#[cfg(feature = "trace")]
#[derive(Clone)]
pub struct Trace {
    logger: Rc<RefCell<dyn Fn(usize)>>,
}

#[cfg(feature = "trace")]
impl Trace {
    pub fn empty() -> Self {
        Self {
            logger: Rc::new(RefCell::new(|_| {})),
        }
    }

    pub fn new<F>(handler: F) -> Self
    where
        F: Fn(usize) + 'static,
    {
        Self {
            logger: Rc::new(RefCell::new(handler)),
        }
    }

    pub fn log(&self) {
        let count = STACK.with(|x| unsafe { &*x.get() }.len());
        let bump = BUMP.with(|x| x.get());
        (*self.logger.borrow())(count + bump);
    }

    pub fn indent() {
        BUMP.with(|x| {
            x.set(x.get() + 1);
        });
    }

    pub fn dedent() {
        BUMP.with(|x| {
            x.set(x.get() - 1);
        });
    }
}

#[cfg(not(feature = "trace"))]
#[derive(Clone)]
pub struct Trace;

#[cfg(not(feature = "trace"))]
impl Trace {
    #[inline]
    pub fn empty() -> Self {
        Self
    }

    #[inline]
    pub fn new<F>(_: F) -> Self
    where
        F: Fn(usize) + 'static,
    {
        Self
    }

    #[inline]
    pub fn log(&self) {}

    #[inline]
    pub fn indent() {}

    #[inline]
    pub fn dedent() {}
}
