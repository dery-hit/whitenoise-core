use crate::errors::*;


use std::collections::HashMap;


use crate::components::{Sensitivity};
use crate::{proto, base};

use crate::components::{Component, Expandable};
use crate::base::{Value, SensitivitySpace, ValueProperties};
use crate::utilities::{prepend, expand_mechanism, broadcast_privacy_usage, get_epsilon};


impl Component for proto::ExponentialMechanism {
    fn propagate_property(
        &self,
        privacy_definition: &proto::PrivacyDefinition,
        _public_arguments: &HashMap<String, Value>,
        properties: &base::NodeProperties,
    ) -> Result<ValueProperties> {
        let mut data_property = properties.get("data")
            .ok_or("data: missing")?.array()
            .map_err(prepend("data:"))?.clone();

        let aggregator = data_property.aggregator.clone()
            .ok_or_else(|| Error::from("aggregator: missing"))?;

        // sensitivity must be computable
        let sensitivity_values = aggregator.component.compute_sensitivity(
            &privacy_definition,
            &aggregator.properties,
            &SensitivitySpace::Exponential)?;

        let sensitivities = sensitivity_values.array()?.f64()?;


        if self.privacy_usage.len() == 0 {
            data_property.releasable = false;
        } else {
            let usages = broadcast_privacy_usage(&self.privacy_usage, sensitivities.len())?;
            let epsilons = usages.iter().map(get_epsilon).collect::<Result<Vec<f64>>>()?;

            // epsilons must be greater than 0.
            for epsilon in epsilons.into_iter() {
                if epsilon <= 0.0 {
                    return Err("epsilon: privacy parameter epsilon must be greater than 0".into());
                };
                if epsilon > 1.0 {
                    println!("Warning: A large privacy parameter of epsilon = {} is in use", epsilon.to_string());
                }
            }
            data_property.releasable = true;
        }

        Ok(data_property.into())
    }
}


impl Expandable for proto::ExponentialMechanism {
    fn expand_component(
        &self,
        _privacy_definition: &proto::PrivacyDefinition,
        component: &proto::Component,
        properties: &base::NodeProperties,
        component_id: &u32,
        maximum_id: &u32,
    ) -> Result<proto::ComponentExpansion> {
        let mut expansion = expand_mechanism(
            &SensitivitySpace::Exponential,
            _privacy_definition,
            component,
            properties,
            component_id,
            maximum_id
        )?;

        let modified_component = component.clone();
        expansion.computation_graph.insert(*component_id, modified_component);
        Ok(expansion)
    }
}
