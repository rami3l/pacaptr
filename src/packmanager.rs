mod homebrew;
mod unknown;

pub use self::{homebrew::Homebrew, unknown::Unknown};
use crate::error::Error;
use crate::exec::{self, Mode};

macro_rules! make_pm {
    ($( $method:ident ), *) => {
        $(fn $method(&self, _kws: &[&str]) -> Result<(), Error> {
            Err(format!("\"{}\" unimplemented", stringify!($method)).into())
        })*
    };
}

pub trait PackManager {
    /// A helper method to simplify direction command invocation.
    /// Override this to implement features such as `dryrun`.
    fn just_run(&self, cmd: &str, subcmd: &[&str], kws: &[&str]) -> Result<(), Error> {
        exec::exec(cmd, subcmd, kws, Mode::CheckErr)?;
        Ok(())
    }

    make_pm!(
        q, qc, qe, qi, qk, ql, qm, qo, qp, qs, qu, r, rn, rns, rs, s, sc, scc, sccc, sg, si, sii,
        sl, ss, su, suy, sw, sy, u
    );
}
