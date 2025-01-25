use rand::{thread_rng, Rng};
use rand_distr::{Distribution, Normal};
use rayon::prelude::*;

use crate::neuron::{Neuron, SynapseType};

pub struct WorldState {
    pub neurons: Vec<Neuron>,
    pub action_potentials: Vec<Vec<bool>>,
    pub time_step: usize,
}

impl WorldState {
    pub fn new(excitatory: usize, inhibitory: usize) -> Self {
        let n = excitatory + inhibitory;

        let mut neurons = vec![];
        for _ in 0..excitatory {
            neurons.push(Neuron::from_synapse_type(SynapseType::Excitatory));
        }
        for _ in 0..inhibitory {
            neurons.push(Neuron::from_synapse_type(SynapseType::Inhibitory));
        }

        let mut connection_matrix = vec![vec![0.; n]; n];
        let mut rng = rand::thread_rng();
        for (i, row) in connection_matrix.iter_mut().enumerate() {
            for (j, cell) in row.iter_mut().enumerate() {
                // don't allow self-connections
                if i != j {
                    if j < excitatory {
                        *cell = 0.5 * rng.gen::<f64>();
                    } else {
                        // inhibitory connections actually inhibit
                        *cell = -rng.gen::<f64>();
                    }
                }
            }
        }

        neurons.iter_mut().enumerate().for_each(|(i, neuron)| {
            neuron.connect(connection_matrix[i].clone());
        });

        WorldState {
            neurons,
            action_potentials: vec![vec![false; n]],
            time_step: 0,
        }
    }

    pub fn step(&mut self) {
        let thalamic_generator = Normal::new(0., 1.).unwrap();
        let thalamic_input: Vec<f64> = thalamic_generator
            .sample_iter(&mut thread_rng())
            .take(self.neurons.len())
            .collect();

        // Get synaptic input once before parallel processing
        let synaptic_input = self.action_potentials[self.time_step].clone();

        // Process neurons in parallel
        let spikes: Vec<bool> = self
            .neurons
            .par_iter_mut() // Parallel mutable iterator
            .zip(thalamic_input.par_iter()) // Parallel immutable iterator
            .map(|(neuron, i)| neuron.step(*i, &synaptic_input))
            .collect();

        self.time_step += 1;
        self.action_potentials.push(spikes);
    }
}
