fn publish() {
    let maybe_publish_request = session.execute(|session| session.extract_publish_request());
    let PublishRequest {
        destination,
        bundle,
        expected_modules,
        allowed_deps,
        check_compat: _,
    } = maybe_publish_request.expect("Publish request exists");
}

fn validate_publish_request(
    &self,
    module_storage: &impl CedraModuleStorage,
    modules: &[CompiledModule],
    mut expected_modules: BTreeSet<String>,
    allowed_deps: Option<BTreeMap<AccountAddress, BTreeSet<String>>>,
) -> VMResult<()> {
    self.reject_unstable_bytecode(modules)?;
    native_validation::validate_module_natives(modules)?;

    for m in modules {
        if !expected_modules.remove(m.self_id().name().as_str()) {
            return Err(Self::metadata_validation_error(&format!(
                "unregistered module: '{}'",
                m.self_id().name()
            )));
        }
        if let Some(allowed) = &allowed_deps {
            for dep in m.immediate_dependencies() {
                if !allowed
                    .get(dep.address())
                    .map(|modules| modules.contains("") || modules.contains(dep.name().as_str()))
                    .unwrap_or(false)
                {
                    return Err(Self::metadata_validation_error(&format!(
                        "unregistered dependency: '{}'",
                        dep
                    )));
                }
            }
        }
        verify_module_metadata_for_module_publishing(m, self.features())
            .map_err(|err| Self::metadata_validation_error(&err.to_string()))?;
    }

    resource_groups::validate_resource_groups(
        module_storage,
        modules,
        self.features()
            .is_enabled(FeatureFlag::SAFER_RESOURCE_GROUPS),
    )?;
    event_validation::validate_module_events(module_storage, modules)?;

    if !expected_modules.is_empty() {
        return Err(Self::metadata_validation_error(
            "not all registered modules published",
        ));
    }
    Ok(())
}
