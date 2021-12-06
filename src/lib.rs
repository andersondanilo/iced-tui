#![feature(iter_zip)]
mod application;
mod constants;
mod renderer;

pub use application::*;
pub use renderer::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
