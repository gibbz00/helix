use crate::{Document, Editor};
use anyhow::{anyhow, bail};
use helix_core::syntax::debug::{
    DebugAdapterConfig, DebugAdapterName, DebugTemplateConfig, DebugTemplateName,
};

pub trait EditorDebugConfig {
    fn select_debug_adapter<'names, 'configs>(
        &'configs self,
        document: &'names Document,
    ) -> anyhow::Result<(&'names DebugAdapterName, &'configs DebugAdapterConfig)>;

    fn get_debug_templates(
        &self,
        debug_adapter_name: &DebugAdapterName,
    ) -> anyhow::Result<Vec<(&DebugTemplateName, &DebugTemplateConfig)>>;
}

impl EditorDebugConfig for Editor {
    fn select_debug_adapter<'names, 'configs>(
        &'configs self,
        document: &'names Document,
    ) -> anyhow::Result<(&'names DebugAdapterName, &'configs DebugAdapterConfig)> {
        let Some(debug_adapter_names) = document
            .language_config()
            .map(|config| &config.debug_adapter_names)
        else {
            bail!("No debug adapter available for language");
        };

        // TEMP: add picker for selecting debug adapter
        let Some(debug_adapter_name) = debug_adapter_names.first() else {
            bail!("No debug adapter listed in debug-adapter");
        };

        let debug_adapter_config = self
            .syn_loader
            .debug_adapter_configs
            .get(debug_adapter_name)
            .ok_or_else(|| {
                anyhow!(
                    "No debug adapter configuration for {}",
                    debug_adapter_name.as_str()
                )
            })?;

        Ok((debug_adapter_name, debug_adapter_config))
    }

    fn get_debug_templates(
        &self,
        debug_adapter_name: &DebugAdapterName,
    ) -> anyhow::Result<Vec<(&DebugTemplateName, &DebugTemplateConfig)>> {
        let Some(language_config) = doc!(self).language_config() else {
            bail!("Missing language config.")
        };

        let debug_templates: Vec<(&DebugTemplateName, &DebugTemplateConfig)> = language_config
            .debug_templates
            .iter()
            .filter(|(_, config)| {
                config.debug_adapter_names.is_empty()
                    || config.debug_adapter_names.contains(debug_adapter_name)
            })
            .collect();

        if debug_templates.is_empty() {
            bail!("No debug templates found.");
        }

        Ok(debug_templates)
    }
}
