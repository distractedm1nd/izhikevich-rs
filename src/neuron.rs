use rand::Rng;

#[derive(Clone)]
pub struct NeuronParameters {
    /// Represents the time scale of the recovery variable u.
    pub a: f64,
    /// Represents the sensitivity of the recovery variable u to the subthreshold fluctuations of the membrane potential v.
    pub b: f64,
    /// Represents the after-spike reset value of the membrane potential v caused by the fast high-threshold K+ conductances.
    pub c: f64,
    /// Represents the after-spike reset value of the recovery variable u caused by slow high-threshold Na+ and K+ conductances.
    pub d: f64,
}

#[derive(Clone, Eq, PartialEq)]
pub enum SynapseType {
    Excitatory,
    Inhibitory,
}
pub struct Neuron {
    // Izhikevich morphology parameters
    parameters: NeuronParameters,
    /// Excitatory or inhibitory, used for scaling thalamic input
    pub synapse_type: SynapseType,
    /// Pairwise weights to other neurons
    connection_weights: Vec<f64>,
    /// Membrane potential.
    pub v: f64,
    /// Membrane recovery variable.
    pub u: f64,
}

impl Neuron {
    pub fn from_synapse_type(synapse_type: SynapseType) -> Self {
        let mut rng = rand::thread_rng();

        let params = match synapse_type {
            SynapseType::Excitatory => {
                let r = rng.gen::<f64>();
                // Comes from Regular Spiking (RS), the r parameter is squared
                // to tend to RS type instead of CH type
                NeuronParameters {
                    a: 0.02,
                    b: 0.2,
                    c: -65. + 15. * r * r,
                    d: 8. - 6. * r * r,
                }
            }
            SynapseType::Inhibitory => {
                let r = rng.gen::<f64>();
                // Comes from Fast Spiking (FS)
                NeuronParameters {
                    a: 0.02 + 0.08 * r,
                    b: 0.25 - 0.05 * r,
                    c: -65.,
                    d: 2.,
                }
            }
        };

        Neuron {
            parameters: params.clone(),
            synapse_type,
            connection_weights: vec![],
            v: -65.,
            u: params.b * -65.,
        }
    }

    pub fn from_neuron_type(neuron_type: NeuronType) -> Self {
        let parameters = neuron_type.params();
        let mut rng = rand::thread_rng();

        let synapse_type = neuron_type.into();
        let params = match synapse_type {
            SynapseType::Excitatory => {
                let r = rng.gen::<f64>();
                NeuronParameters {
                    a: parameters.a,
                    b: parameters.b,
                    c: -65. + 15. * r * r,
                    d: 8. - 6. * r * r,
                }
            }
            SynapseType::Inhibitory => {
                let r = rng.gen::<f64>();
                NeuronParameters {
                    a: 0.02 + 0.08 * r,
                    b: 0.25 - 0.05 * r,
                    c: -65.,
                    d: 2.,
                }
            }
        };

        Neuron {
            parameters: params,
            synapse_type,
            connection_weights: vec![],
            v: -65.,
            u: parameters.b * -65.,
        }
    }

    pub fn connect(&mut self, connection_weights: Vec<f64>) {
        self.connection_weights = connection_weights;
    }

    pub fn step(&mut self, thalamic_input: f64, synaptic_input: &[bool]) -> bool {
        // excitatory synapses have stronger strengths to the input
        let mut i = match self.synapse_type {
            SynapseType::Excitatory => thalamic_input * 5.,
            SynapseType::Inhibitory => thalamic_input * 2.,
        };

        i += synaptic_input
            .iter()
            .zip(&self.connection_weights)
            .filter(|&(&spike, _)| spike)
            .map(|(_, weight)| weight)
            .sum::<f64>();

        // in 2 time steps for numerical stability
        self.v += 0.5 * ((0.04 * self.v * self.v) + (5. * self.v) + 140. - self.u + i);
        self.v += 0.5 * ((0.04 * self.v * self.v) + (5. * self.v) + 140. - self.u + i);
        self.u += self.parameters.a * ((self.parameters.b * self.v) - self.u);

        // action potential
        if self.v >= 30. {
            self.v = self.parameters.c;
            self.u += self.parameters.d;
            true
        } else {
            false
        }
    }
}

pub enum NeuronType {
    Regular,
    IntrinsicallyBursting,
    Chattering,
    FastSpiking,
    LowThresholdSpiking,
}

impl From<NeuronType> for SynapseType {
    fn from(neuron_type: NeuronType) -> Self {
        match neuron_type {
            NeuronType::Regular => SynapseType::Excitatory,
            NeuronType::IntrinsicallyBursting => SynapseType::Excitatory,
            NeuronType::Chattering => SynapseType::Excitatory,
            NeuronType::FastSpiking => SynapseType::Inhibitory,
            NeuronType::LowThresholdSpiking => SynapseType::Inhibitory,
        }
    }
}

const REGULAR_PARAMS: NeuronParameters = NeuronParameters {
    a: 0.02,
    b: 0.2,
    c: -65.0,
    d: 2.0,
};

const INTRINSICALLY_BURSTING_PARAMS: NeuronParameters = NeuronParameters {
    a: 0.02,
    b: 0.2,
    c: -55.0,
    d: 4.0,
};

const CHATTERING_PARAMS: NeuronParameters = NeuronParameters {
    a: 0.02,
    b: 0.2,
    c: -50.0,
    d: 2.0,
};

const FAST_SPIKING_PARAMS: NeuronParameters = NeuronParameters {
    a: 0.1,
    b: 0.2,
    c: -65.0,
    d: 2.0,
};

const LOW_THRESHOLD_SPIKING_PARAMS: NeuronParameters = NeuronParameters {
    a: 0.1,
    b: 0.25,
    c: -55.0,
    d: 2.0,
};

impl NeuronType {
    pub fn params(&self) -> NeuronParameters {
        match self {
            NeuronType::Regular => REGULAR_PARAMS,
            NeuronType::IntrinsicallyBursting => INTRINSICALLY_BURSTING_PARAMS,
            NeuronType::Chattering => CHATTERING_PARAMS,
            NeuronType::FastSpiking => FAST_SPIKING_PARAMS,
            NeuronType::LowThresholdSpiking => LOW_THRESHOLD_SPIKING_PARAMS,
        }
    }
}
