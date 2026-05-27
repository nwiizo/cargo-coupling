use std::collections::HashMap;

use crate::metrics::coupling::CouplingMetrics;
use crate::metrics::dimensions::Subdomain;
use crate::metrics::project::ProjectMetrics;

pub(crate) fn build_target_subdomain_map(
    metrics: &ProjectMetrics,
) -> HashMap<String, Option<Subdomain>> {
    let mut targets = HashMap::new();

    for (module_key, module) in &metrics.modules {
        let Some(subdomain) = module.subdomain else {
            continue;
        };

        insert_subdomain_alias(&mut targets, module_key, subdomain);
        insert_subdomain_alias(&mut targets, &module.name, subdomain);

        if let Some(short_name) = module_key.rsplit("::").next() {
            insert_subdomain_alias(&mut targets, short_name, subdomain);
        }
        if let Some(short_name) = module.name.rsplit("::").next() {
            insert_subdomain_alias(&mut targets, short_name, subdomain);
        }
        if let Some(file_stem) = module.path.file_stem().and_then(|stem| stem.to_str()) {
            insert_subdomain_alias(&mut targets, file_stem, subdomain);
        }

        for type_name in module.type_definitions.keys() {
            insert_subdomain_alias(&mut targets, type_name, subdomain);
        }
        for function_name in module.function_definitions.keys() {
            insert_subdomain_alias(&mut targets, function_name, subdomain);
        }
    }

    targets
}

fn insert_subdomain_alias(
    targets: &mut HashMap<String, Option<Subdomain>>,
    alias: &str,
    subdomain: Subdomain,
) {
    if !alias.is_empty() {
        targets
            .entry(alias.to_string())
            .and_modify(|existing| {
                if *existing != Some(subdomain) {
                    *existing = None;
                }
            })
            .or_insert(Some(subdomain));
    }
}

pub(crate) fn coupling_with_essential_volatility(
    coupling: &CouplingMetrics,
    target_subdomains: &HashMap<String, Option<Subdomain>>,
) -> CouplingMetrics {
    let Some(subdomain) = target_subdomain_for_coupling(&coupling.target, target_subdomains) else {
        return coupling.clone();
    };

    let mut effective = coupling.clone();
    effective.volatility = subdomain.expected_volatility();
    effective
}

fn target_subdomain_for_coupling(
    target: &str,
    target_subdomains: &HashMap<String, Option<Subdomain>>,
) -> Option<Subdomain> {
    target_subdomains
        .get(target)
        .copied()
        .flatten()
        .or_else(|| {
            target
                .rsplit("::")
                .find_map(|part| target_subdomains.get(part).copied().flatten())
        })
}
