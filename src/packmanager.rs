mod homebrew;
mod unknown;

pub use self::{homebrew::Homebrew, unknown::Unknown};
use crate::error::Error;

macro_rules! make_pm {
    ($( $method:ident ), *) => {
        $(fn $method(&self, _kws: &[&str]) -> Result<(), Error> {
            Err(format!("\"{}\" unimplemented", stringify!($method)).into())
        })*
    };
}

pub trait PackManager {
    fn run(cmd: &str, kws: &[&str]) -> Result<(), Error> {
        todo!()
    }

    make_pm!(
        q, qc, qe, qi, qk, ql, qm, qo, qp, qs, qu, r, rn, rns, rs, s, sc, scc, sccc, sg, si, sii,
        sl, ss, su, suy, sw, sy, u
    );
}
