// pub trait Wrappable<'a> {
//     type Output;
//     fn wrap(self, client: &'a Client) -> Self::Output
//     where
//         Self::Output: Wrapper<'a, Self>,
//         Self: Sized;
// }

// pub trait Wrapper<'a, T> {
//     fn inner(&self) -> &T;
// }
