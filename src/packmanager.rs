mod homebrew;
mod unknown;

macro_rules! make_pm {
    ($( $method:ident ), *) => {
        $(fn $method(&self, _kws: &[&str]) -> Result<(), String> {
            unimplemented!("{}", stringify!($method))
        })*
    };
}

pub trait PackManager {
    make_pm!(
        q, qc, qe, qi, qk, ql, qm, qo, qp, qs, qu, r, rn, rns, rs, s, sc, scc, sccc, sg, si, sii,
        sl, ss, su, suy, sw, sy, u
    );
}
