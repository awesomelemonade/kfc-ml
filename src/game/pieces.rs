core!();

use std::convert::{TryFrom, TryInto};

pub use super::*;

#[derive(Debug, Clone)]
pub struct Pieces<const N: usize> {
    pieces: [Option<Piece>; N], // TODO: split by side?
    length: usize,
}

impl<const N: usize> Pieces<N> {
    pub fn iter(&self) -> PiecesIter<'_, N> {
        PiecesIter::new(self)
    }

    // TODO: likely needs something more complicated
    // https://doc.rust-lang.org/src/core/slice/iter.rs.html#187
    pub fn iter_mut(&mut self) -> PiecesIterMut<'_, N> {
        PiecesIterMut::new(self)
    }

    pub fn len(&self) -> usize {
        self.length
    }
}

pub struct PiecesIter<'a, const N: usize> {
    pieces: &'a Pieces<N>,
    index: usize,
}

impl<'a, const N: usize> PiecesIter<'a, N> {
    pub fn new(pieces: &'a Pieces<N>) -> Self {
        Self { pieces, index: 0 }
    }
}

impl<'a, const N: usize> Iterator for PiecesIter<'a, N> {
    type Item = &'a Piece;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < N && self.pieces.pieces[self.index].is_none() {
            self.index += 1;
        }
        if self.index == N {
            None
        } else {
            let result = &self.pieces.pieces[self.index];
            self.index += 1;
            Some(result.as_mut().unwrap())
        }
    }
}

pub struct PiecesIterMut<'a, const N: usize> {
    pieces: &'a mut Pieces<N>,
    index: usize,
}

impl<'a, const N: usize> PiecesIterMut<'a, N> {
    pub fn new(pieces: &'a mut Pieces<N>) -> Self {
        Self { pieces, index: 0 }
    }
}

impl<'a, const N: usize> Iterator for PiecesIterMut<'a, N> {
    type Item = &'a mut Piece;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < N && self.pieces.pieces[self.index].is_none() {
            self.index += 1;
        }
        if self.index == N {
            None
        } else {
            let result = &mut self.pieces.pieces[self.index];
            self.index += 1;
            Some(result.as_mut().unwrap())
        }
    }
}

impl<const N: usize> TryFrom<Vec<Piece>> for Pieces<N> {
    type Error = Vec<Piece>;

    fn try_from(vec: Vec<Piece>) -> Result<Self, Self::Error> {
        let pieces: [Piece; N] = vec.try_into()?;
        let pieces = pieces.map(Some);
        Ok(Self {
            pieces,
            length: pieces.len(),
        })
    }
}
