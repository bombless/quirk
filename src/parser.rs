use types::*;
use std::ops::*;

pub struct Operation<T: ?Sized>(Box<T>);

pub fn one_char(c: char) -> Operation<Fn(&mut TwoWay)-> Result<(), ()>> {
	Operation(Box::new(move |s| {
		let ptr = s.ptr();
		if Some(c) == s.read() {
			Ok(())
		} else {
			s.set(ptr);
			Err(())
		}
	}))
}

impl<R, T: ?Sized + Fn(&mut TwoWay) -> R> Operation<T> {
	pub fn call(&self, s: &mut TwoWay) -> R {
		self.0(s)
	}
}

pub fn add<P: ?Sized, Q: ?Sized, U, V, E>(lhs: Operation<P>, rhs: Operation<Q>) -> Operation<Fn(&mut TwoWay)->Result<(U, V), E>>
	where
		P: Fn(&mut TwoWay) -> Result<U, E> + 'static,
		Q: Fn(&mut TwoWay) -> Result<V, E> + 'static {
	Operation(Box::new(move |s| {
		let ptr = s.ptr();
		match lhs.0(s) {
			Ok(p) => match rhs.0(s) {
				Ok(q) => Ok((p, q)),
				Err(e) => {
					s.set(ptr);
					Err(e)
				}
			},
			Err(e) => {
				s.set(ptr);
				Err(e)
			}
		}
	}))
}

impl<P: ?Sized, Q: ?Sized, U, V, E> Add<Operation<Q>> for Operation<P>
	where
		P: Fn(&mut TwoWay) -> Result<U, E> + 'static,
		Q: Fn(&mut TwoWay) -> Result<V, E> + 'static {
	type Output = Operation<Fn(&mut TwoWay)->Result<(U, V), E>>;
	fn add(self, rhs: Operation<Q>) -> Operation<Fn(&mut TwoWay)->Result<(U, V), E>> {
		Operation(Box::new(move |s| {
			let ptr = s.ptr();
			match self.0(s) {
				Ok(p) => match rhs.0(s) {
					Ok(q) => Ok((p, q)),
					Err(e) => {
						s.set(ptr);
						Err(e)
					}
				},
				Err(e) => {
					s.set(ptr);
					Err(e)
				}
			}
		}))
	}
}