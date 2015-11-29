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

pub trait Comment {
	fn comment_before<R, T: ?Sized + Fn(&mut TwoWay) -> R + 'static>(&self, op: Operation<T>) -> Operation<Fn(&mut TwoWay) -> R>;
	fn comment_after<R, T: ?Sized + Fn(&mut TwoWay) -> R + 'static>(&self, op: Operation<T>) -> Operation<Fn(&mut TwoWay) -> R>;
}

impl Comment for str {
	fn comment_before<R, T: ?Sized + Fn(&mut TwoWay) -> R + 'static>(&self, op: Operation<T>) -> Operation<Fn(&mut TwoWay) -> R> {
		let c = self.to_string();
		let a = move || println!("{}", c);
		before(op, a)
	}
	fn comment_after<R, T: ?Sized + Fn(&mut TwoWay) -> R + 'static>(&self, op: Operation<T>) -> Operation<Fn(&mut TwoWay) -> R> {
		let c = self.to_string();
		let a = move || println!("{}", c);
		after(op, a)
	}
}

pub fn comment<R, T: ?Sized + Fn(&mut TwoWay) -> R + 'static>(op: Operation<T>, c: &str) -> Operation<Fn(&mut TwoWay) -> R> {
	let c = c.to_string();
	Operation(Box::new(move |s| {
		println!("{}", c);
		op.call(s)
	}))
}

pub fn after<Action: Fn() + 'static, R, T: ?Sized + Fn(&mut TwoWay) -> R + 'static>(op: Operation<T>, a: Action) -> Operation<Fn(&mut TwoWay) -> R> {
	Operation(Box::new(move |s| {
		a();
		op.call(s)
	}))
}

pub fn before<Action: Fn() + 'static, R, T: ?Sized + Fn(&mut TwoWay) -> R + 'static>(op: Operation<T>, a: Action) -> Operation<Fn(&mut TwoWay) -> R> {
	Operation(Box::new(move |s| {
		let ret = op.call(s);
		a();
		ret
	}))
}

impl<R, T: ?Sized + Fn(&mut TwoWay) -> R> Operation<T> {
	pub fn call(&self, s: &mut TwoWay) -> R {
		self.0(s)
	}
	
	pub fn comment(self, c: &str) -> Operation<Fn(&mut TwoWay) -> R>
		where T: 'static {
		comment(self, c)
	}
	
	pub fn before<Action: Fn() + 'static>(self, a: Action) -> Operation<Fn(&mut TwoWay) -> R>
		where T: 'static {
		before(self, a)
	}
	
	pub fn after<Action: Fn() + 'static>(self, a: Action) -> Operation<Fn(&mut TwoWay) -> R>
		where T: 'static {
		after(self, a)
	}
}

impl<'a, R, T: ?Sized + Fn(&'a mut TwoWay) -> R> BitOr<&'a mut TwoWay> for Operation<T> {
	type Output = R;
	fn bitor(self, rhs: &'a mut TwoWay) -> R {
		self.0(rhs)
	}
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