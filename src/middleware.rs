//! Middleware allow changing TextBox behaviour.

use core::{
    cell::{Cell, RefCell, RefMut},
    hash::{Hash, Hasher},
    marker::PhantomData,
};
use embedded_graphics::{
    draw_target::DrawTarget,
    prelude::{PixelColor, Point},
    primitives::Rectangle,
    text::renderer::TextRenderer,
};

use crate::parser::Token;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(crate) enum ProcessingState {
    Measure,
    Render,
}

pub trait Middleware<'a, C>: Clone
where
    C: PixelColor,
{
    /// Called when a new line is started.
    #[inline]
    fn new_line(&mut self) {}

    #[inline]
    fn next_token_to_measure(
        &mut self,
        next_token: &mut impl Iterator<Item = Token<'a>>,
    ) -> Option<Token<'a>> {
        next_token.next()
    }

    #[inline]
    fn next_token_to_render(
        &mut self,
        next_token: &mut impl Iterator<Item = Token<'a>>,
    ) -> Option<Token<'a>> {
        next_token.next()
    }

    #[inline]
    fn post_render<T, D>(
        &mut self,
        _draw_target: &mut D,
        _character_style: &T,
        _text: &str,
        _bounds: Rectangle,
    ) -> Result<(), D::Error>
    where
        T: TextRenderer<Color = C>,
        D: DrawTarget<Color = C>,
    {
        Ok(())
    }

    #[inline]
    fn post_line_start<T, D>(
        &mut self,
        _draw_target: &mut D,
        _character_style: &T,
        _pos: Point,
    ) -> Result<(), D::Error>
    where
        T: TextRenderer<Color = C>,
        D: DrawTarget<Color = C>,
    {
        Ok(())
    }
}

#[derive(Clone, Copy, Default)]
pub struct NoMiddleware<C>(PhantomData<C>);

impl<C> NoMiddleware<C> {
    #[inline]
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<'a, C> Middleware<'a, C> for NoMiddleware<C> where C: PixelColor {}

#[derive(Clone, Debug)]
pub(crate) struct MiddlewareWrapper<'a, M, C> {
    pub middleware: RefCell<M>,
    state: Cell<ProcessingState>,
    measurement_token: RefCell<Option<Token<'a>>>,
    render_token: RefCell<Option<Token<'a>>>,
    _marker: PhantomData<C>,
}

impl<'a, M, C> Hash for MiddlewareWrapper<'a, M, C> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.state.get().hash(state)
    }
}

impl<'a, M, C> MiddlewareWrapper<'a, M, C>
where
    C: PixelColor,
    M: Middleware<'a, C>,
{
    pub fn new(middleware: M) -> Self {
        Self {
            _marker: PhantomData,
            middleware: RefCell::new(middleware),
            state: Cell::new(ProcessingState::Measure),
            measurement_token: RefCell::new(None),
            render_token: RefCell::new(None),
        }
    }

    pub fn new_line(&self) {
        self.middleware.borrow_mut().new_line();
    }

    pub fn set_state(&self, state: ProcessingState) {
        self.state.set(state);
    }

    fn current_token_ref(&self) -> RefMut<Option<Token<'a>>> {
        match self.state.get() {
            ProcessingState::Measure => self.measurement_token.borrow_mut(),
            ProcessingState::Render => self.render_token.borrow_mut(),
        }
    }

    fn next_token(&self, next_token: &mut impl Iterator<Item = Token<'a>>) -> Option<Token<'a>> {
        let mut mw = self.middleware.borrow_mut();
        match self.state.get() {
            ProcessingState::Measure => mw.next_token_to_measure(next_token),
            ProcessingState::Render => mw.next_token_to_render(next_token),
        }
    }

    pub fn peek_token(
        &self,
        next_token: &mut impl Iterator<Item = Token<'a>>,
    ) -> Option<Token<'a>> {
        let mut peeked = self.current_token_ref();
        if peeked.is_none() {
            *peeked = self.next_token(next_token);
        }
        peeked.clone()
    }

    pub fn consume_peeked_token(&self) {
        self.current_token_ref().take();
    }

    pub fn replace_peeked_token(&self, token: Token<'a>) {
        self.current_token_ref().replace(token);
    }
}
