use std::collections::HashMap;

use super::{CouplingMetrics, Distance, ModuleMetrics};

/// Exact module identities take precedence over unique short names and item aliases.
pub(crate) struct ModuleIndex<'a> {
    modules: &'a HashMap<String, ModuleMetrics>,
    members: &'a [String],
    aliases: HashMap<&'a str, Option<&'a ModuleMetrics>>,
}

impl<'a> ModuleIndex<'a> {
    pub fn new(modules: &'a HashMap<String, ModuleMetrics>, members: &'a [String]) -> Self {
        let mut aliases = HashMap::new();
        for (key, module) in modules {
            let names = [
                key.rsplit("::").next(),
                module.name.rsplit("::").next(),
                module.path.file_stem().and_then(|stem| stem.to_str()),
            ];
            for name in names
                .into_iter()
                .flatten()
                .chain(module.type_definitions.keys().map(String::as_str))
                .chain(module.function_definitions.keys().map(String::as_str))
            {
                aliases
                    .entry(name)
                    .and_modify(|existing: &mut Option<&ModuleMetrics>| {
                        if existing.is_some_and(|previous| !std::ptr::eq(previous, module)) {
                            *existing = None;
                        }
                    })
                    .or_insert(Some(module));
            }
        }
        Self {
            modules,
            members,
            aliases,
        }
    }

    pub fn resolve(&self, target: &str) -> Option<&'a ModuleMetrics> {
        let target = target.strip_prefix("crate::").unwrap_or(target);
        let crate_prefix = target
            .split_once("::")
            .map(|(prefix, _)| prefix)
            .filter(|prefix| self.members.iter().any(|name| name == prefix));
        let mut candidate = target;
        loop {
            if let Some(module) = self.modules.get(candidate) {
                return Some(module);
            }
            if let Some((prefix, local)) = candidate.split_once("::")
                && Some(prefix) == crate_prefix
                && let Some(module) = self.modules.get(local)
            {
                return Some(module);
            }
            let Some((parent, _)) = candidate.rsplit_once("::") else {
                break;
            };
            candidate = parent;
        }
        let local = crate_prefix.and_then(|_| target.split_once("::").map(|(_, rest)| rest));
        let name = local.unwrap_or(target);
        // Metrics built through the public API can have crate-qualified endpoints
        // without workspace metadata. Only accept an exact module in that case.
        if self.members.is_empty()
            && let Some((_, rest)) = name.split_once("::")
            && let Some(module) = self.modules.get(rest)
        {
            return Some(module);
        }
        // An associated item inherits the owner of its uniquely resolved type.
        let owner = name.split("::").next().unwrap_or(name);
        self.aliases.get(owner).copied().flatten()
    }

    pub fn provider(&self, edge: &CouplingMetrics) -> Option<&'a ModuleMetrics> {
        if edge.distance == Distance::DifferentCrate {
            return None;
        }
        self.resolve(&edge.target)
    }
}
