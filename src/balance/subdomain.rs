use std::borrow::Cow;

use crate::metrics::coupling::CouplingMetrics;
use crate::metrics::target::ModuleIndex;

impl ModuleIndex<'_> {
    pub(crate) fn effective_coupling<'a>(
        &self,
        edge: &'a CouplingMetrics,
    ) -> Cow<'a, CouplingMetrics> {
        let mut effective = Cow::Borrowed(edge);
        if let Some(subdomain) = self.provider(edge).and_then(|module| module.subdomain) {
            let volatility = subdomain.expected_volatility();
            if volatility != edge.volatility {
                effective.to_mut().volatility = volatility;
            }
        }
        effective
    }
}
