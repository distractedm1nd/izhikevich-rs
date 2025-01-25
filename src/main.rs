mod neuron;
mod simulator;

use clap::Parser;
use plotters::prelude::*;
use simulator::WorldState;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Number of excitatory neurons
    #[arg(short, long, default_value_t = 800)]
    excitatory: usize,

    /// Number of inhibitory neurons
    #[arg(short, long, default_value_t = 200)]
    inhibitory: usize,

    /// Simulation duration in milliseconds
    #[arg(short, long, default_value_t = 1000)]
    milliseconds: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let total_neurons = args.excitatory + args.inhibitory;

    let mut world_state = WorldState::new(args.excitatory, args.inhibitory);
    for t in 0..args.milliseconds {
        if t % 100 == 0 {
            println!("Time step: {}", t);
        }
        world_state.step();
    }

    let mut spikes = Vec::new();
    for (time_step, step_spikes) in world_state.action_potentials.iter().enumerate() {
        for (neuron_idx, &spiked) in step_spikes.iter().enumerate() {
            if spiked {
                spikes.push((time_step as i32, neuron_idx as i32));
            }
        }
    }

    let root = BitMapBackend::new("spikes.png", (800, 1200)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .margin(20)
        .x_label_area_size(35)
        .y_label_area_size(35)
        .build_cartesian_2d(0..args.milliseconds as i32, 0..total_neurons as i32)?;

    chart
        .configure_mesh()
        .disable_mesh()
        .x_desc("Time (ms)")
        .y_desc("Neuron Index")
        .draw()?;

    chart.draw_series(
        spikes
            .iter()
            .map(|&(x, y)| Circle::new((x, y), 1, RGBAColor(0, 0, 0, 0.3).filled())),
    )?;

    root.present()?;
    Ok(())
}
