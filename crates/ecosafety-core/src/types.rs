// crates/ecosafety-core/src/types.rs
#[derive(Clone, Copy, Debug)]
pub struct Residual<const N: usize> {
    pub r: [f64; N],   // normalized risk coords
    pub w: [f64; N],   // weights from CorridorBands
    pub vt: f64,       // Lyapunov residual V_t
}

impl<const N: usize> Residual<N> {
    #[inline]
    pub fn recompute_vt(&mut self) {
        let mut acc = 0.0;
        let mut i = 0;
        while i < N {
            let ri = self.r[i];
            let wi = self.w[i];
            acc += wi * ri * ri;
            i += 1;
        }
        self.vt = acc;
    }
}

// Example: Phoenix canal node with 7 fixed coordinates
pub type CanalResidual = Residual<7>;
