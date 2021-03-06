use types::*;
use std::ops::*;

pub struct Operation<T: ?Sized>(Box<T>);

pub type ParseOperation<R> = Operation<Fn(&mut TwoWay) -> R>;

pub fn one_char(c: char) -> ParseOperation<Result<(), ()>> {
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

impl<U, E> ParseOperation<Result<U, E>> {
	pub fn map<V, F: Fn(U)->V + 'static>(self, map: F) -> ParseOperation<Result<V, E>> {
		Operation(Box::new(move |s| {
			match self.call(s) {
				Ok(u) => Ok(map(u)),
				Err(err) => Err(err)
			}
		}))
	}
}

pub fn repeat_until<R> (o: ParseOperation<Result<R, ()>>, fail: ParseOperation<Result<(), ()>>) -> ParseOperation<Result<Vec<R>, ()>> {
	Operation(Box::new(move |s| {
		let ptr = s.ptr();
		let mut ret = Vec::new();
		while fail.call(s).is_err() {
			if let Ok(r) = o.call(s) {
				ret.push(r)
			} else {
				s.set(ptr);
				return Err(())
			}
		}
		Ok(ret)
	}))
}

pub fn one_of(vec: Vec<ParseOperation<Result<(), ()>>>) -> ParseOperation<Result<(), ()>> {
	let mut ret: ParseOperation<_> = Operation(Box::new(|_| Err(())));
	for x in vec {
		ret = ret ^ x
	}
	ret
}

pub fn any_char_except(v: Vec<char>) -> ParseOperation<Result<(), ()>> {
	Operation(Box::new(move |s| {
		let v = &v;
		let ptr = s.ptr();
		if s.read().map_or(true, |c| v.into_iter().any(|&x| x == c)) {
			s.set(ptr);
			Err(())
		} else {
			Ok(())
		}
	}))
}

pub fn take_while(op: ParseOperation<Result<(), ()>>) -> ParseOperation<Result<String, ()>> {
	Operation(Box::new(move |s| {
		let start_ptr = s.ptr();
		let mut end_ptr = start_ptr;
		while op.call(s).is_ok() {
			end_ptr = s.ptr();
		}
		s.set(start_ptr);
		let mut ret = String::new();
		for _ in start_ptr .. end_ptr {
			ret.push(s.read().unwrap())
		}
		s.set(end_ptr);
		if ret.len() > 0 {
			Ok(ret)
		} else {
			Err(())
		}
	}))
}

pub fn range(r: Range<char>) -> ParseOperation<Result<(), ()>> {
	Operation(Box::new(move |s| {
		let ptr = s.ptr();
		match s.read() {
			Some(c) if c >= r.start && c <= r.end => {
				Ok(())
			}
			_ => {
				s.set(ptr);
				Err(())
			}
		}
	}))
}

pub fn plain(s: &str) -> ParseOperation<Result<(), ()>> {
	let mut ret: ParseOperation<Result<(), ()>> = Operation(Box::new(|_| Ok(())));
	for x in s.chars() {
		ret = ret + one_char(x) >> ();
	}
	ret
}

pub trait Func<R>: Fn(&mut TwoWay) -> R + 'static {}

impl <R, T: ?Sized + Fn(&mut TwoWay) -> R + 'static> Func<R> for T {}

pub trait Comment {
	fn comment_before<R, T: ?Sized + Func<R>>(&self, Operation<T>) -> ParseOperation<R>;
	fn comment_after<R, T: ?Sized + Func<R>>(&self, Operation<T>) -> ParseOperation<R>;
}

impl Comment for str {
	fn comment_before<R, T: ?Sized + Func<R>>(&self, op: Operation<T>) -> ParseOperation<R> {
		let c = self.to_string();
		let a = move || println!("{}", c);
		after(op, a)
	}
	
	fn comment_after<R, T: ?Sized + Func<R>>(&self, op: Operation<T>) -> ParseOperation<R> {
		let c = self.to_string();
		let a = move || println!("{}", c);
		before(op, a)
	}
}

pub fn comment<R, T: ?Sized + Func<R>>(op: Operation<T>, c: &str) -> ParseOperation<R> {
	let c = c.to_string();
	Operation(Box::new(move |s| {
		println!("{}", c);
		op.call(s)
	}))
}

pub fn after<Action: Fn() + 'static, R, T: ?Sized + Func<R>>(op: Operation<T>, a: Action) -> ParseOperation<R> {
	Operation(Box::new(move |s| {
		a();
		op.call(s)
	}))
}

pub fn before<Action: Fn() + 'static, R, T: ?Sized + Func<R>>(op: Operation<T>, a: Action) -> ParseOperation<R> {
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
	
	pub fn from(v: Box<T>) -> Self {
		Operation(v)
	}
	
	pub fn comment(self, c: &str) -> ParseOperation<R>
		where T: 'static {
		comment(self, c)
	}
	
	pub fn before<Action: Fn() + 'static>(self, a: Action) -> ParseOperation<R>
		where T: 'static {
		before(self, a)
	}
	
	pub fn after<Action: Fn() + 'static>(self, a: Action) -> ParseOperation<R>
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

pub fn drop_first<U, V, E>(po: ParseOperation<Result<(U, V), E>>) -> ParseOperation<Result<V, E>> {
	Operation(Box::new(move |s| {
		match po.call(s) {
			Ok((_, ok)) => Ok(ok),
			Err(err) => Err(err)
		}
	}))
}

pub fn drop_second<U, V, E>(po: ParseOperation<Result<(U, V), E>>) -> ParseOperation<Result<U, E>> {
	Operation(Box::new(move |s| {
		match po.call(s) {
			Ok((ok, _)) => Ok(ok),
			Err(err) => Err(err)
		}
	}))
}

impl<P: ?Sized, Q: ?Sized, U, V, E> Add<Operation<Q>> for Operation<P>
	where
		P: Fn(&mut TwoWay) -> Result<U, E> + 'static,
		Q: Fn(&mut TwoWay) -> Result<V, E> + 'static {
	type Output = Operation<Fn(&mut TwoWay)->Result<(U, V), E>>;
	fn add(self, rhs: Operation<Q>) -> ParseOperation<Result<(U, V), E>> {
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

impl<P: ?Sized, Q: ?Sized, T, E> BitXor<Operation<Q>> for Operation<P>
	where
		P: Fn(&mut TwoWay) -> Result<T, E> + 'static,
		Q: Fn(&mut TwoWay) -> Result<T, E> + 'static {
	type Output = Operation<Fn(&mut TwoWay)->Result<T, E>>;
	fn bitxor(self, rhs: Operation<Q>) -> ParseOperation<Result<T, E>> {
		Operation(Box::new(move |s| {
			let ptr = s.ptr();
			match self.0(s) {
				Ok(p) => Ok(p),
				Err(e) => {
					s.set(ptr);
					match rhs.0(s) {
						Ok(q) => Ok(q),
						Err(_) => {
							s.set(ptr);
							Err(e)
						}
					}
				}
			}
		}))
	}
}

pub fn twice<R, R2>(op: ParseOperation<Result<R, ()>>, middle: ParseOperation<Result<R2, ()>>) -> ParseOperation<Result<(R, R2), ()>> {
	Operation(Box::new(move |s| {
		let ptr = s.ptr();
		if let Ok(r) = op.call(s) {
			let ptr_m = s.ptr();
			if let Ok(r2) = middle.call(s) {
				if op.call(s).is_ok() && s.repeating(ptr, ptr_m) {
					return Ok((r, r2))
				}
			}
		}
		s.set(ptr);
		Err(())
	}))
}

pub fn ignore<Ok, Err, T: ?Sized + Func<Result<Ok, Err>>, Q: Clone + 'static>(op: Operation<T>, to: Q) -> ParseOperation<Result<Q, Err>> {
	let clone = move || to.clone();
	Operation(Box::new(move |s| {
		match op.call(s) {
			Ok(_) => Ok(clone()),
			Err(err) => Err(err)
		}
	}))
}

impl<Ok, Err, T: ?Sized + Fn(&mut TwoWay) -> Result<Ok, Err> + 'static, Q: Clone + 'static> Shr<Q> for Operation<T> {
	type Output = ParseOperation<Result<Q, Err>>;
	fn shr(self, rhs: Q) -> ParseOperation<Result<Q, Err>> {
		ignore(self, rhs)
	}
}
