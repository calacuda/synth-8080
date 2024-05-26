use dot_writer::{Attributes, Color, DotWriter, Scope, Shape};
// use graphviz_rust::dot_generator::*;
// use graphviz_rust::dot_structures::*;
use std::fmt::Display;
use std::{ops::Index, sync::Arc};
use synth_8080::chorus;
use synth_8080::common::Module;
use synth_8080::delay;
use synth_8080::echo;
use synth_8080::envelope;
use synth_8080::lfo;
use synth_8080::midi_osc;
use synth_8080::output;
use synth_8080::overdrive;
use synth_8080::reverb;
use synth_8080::vco;
use synth_8080_lib::ModuleType;

type Counter = u8;

#[derive(Debug, Clone)]
pub struct ModCounter {
    vco: Counter,
    // vco_i: Counter,
    lfo: Counter,
    // lfo_i: Counter,
    mco: Counter,
    // mco_i: Counter,
    echo: Counter,
    // echo_i: Counter,
    delay: Counter,
    // delay_i: Counter,
    chorus: Counter,
    // chorus_i: Counter,
    reverb: Counter,
    // reverd_i: Counter,
    env: Counter,
    // env_i: Counter,
    od: Counter,
    // od_i: Counter,
    output: Counter,
    i: usize,
    modules: Arc<[ModuleType]>,
    // pub base_graph: ModuleBaseGraph,
    pub graph: Vec<u8>,
    // graph_writer: DotWriter<'a>,
}

impl ModCounter {
    fn new(modules: &[ModuleType]) -> Self {
        let mods = modules.to_owned().into();
        // let mut bytes = Vec::new();

        Self {
            vco: 0,
            lfo: 0,
            mco: 0,
            echo: 0,
            delay: 0,
            chorus: 0,
            reverb: 0,
            env: 0,
            od: 0,
            output: 0,
            i: 0,
            modules: mods,
            // base_graph: ModuleBaseGraph::default(),
            graph: Vec::new(),
            // graph_writer: DotWriter::from(&mut bytes),
        }
    }

    fn mk_graph(&mut self) {
        let mut writer = DotWriter::from(&mut self.graph);

        // let module = if self.i < self.modules.len() {
        //     *self.modules.index(self.i)
        // } else {
        //     return None;
        // };

        // self.i += 1;

        let mut graph = writer.digraph();

        for module in self.modules.into_iter() {
            match module {
                ModuleType::Vco => {
                    let count = self.vco;
                    mk_subgraph(
                        count,
                        *module,
                        vco::Vco::get_input_names().into_iter(),
                        vco::Vco::get_output_names().into_iter(),
                        &mut graph.cluster(),
                    );

                    // self.base_graph.vco.push(g.clone());
                    // self.graphs.push(Stmt::Subgraph(g));

                    self.vco += 1;
                    // (module, self.vco)
                }
                ModuleType::Lfo => {
                    let count = self.lfo;
                    mk_subgraph(
                        count,
                        *module,
                        lfo::Lfo::get_input_names().into_iter(),
                        lfo::Lfo::get_output_names().into_iter(),
                        &mut graph.cluster(),
                    );

                    // self.base_graph.lfo.push(g.clone());
                    // self.graphs.push(Stmt::Subgraph(g));

                    self.lfo += 1;
                    // (module, self.lfo)
                }
                ModuleType::MCO => {
                    let count = self.mco;
                    mk_subgraph(
                        count,
                        *module,
                        midi_osc::MidiOsc::get_input_names().into_iter(),
                        midi_osc::MidiOsc::get_output_names().into_iter(),
                        &mut graph.cluster(),
                    );

                    // self.base_graph.mco.push(g.clone());
                    // self.graphs.push(Stmt::Subgraph(g));

                    self.mco += 1;
                    // (module, self.mco)
                }
                ModuleType::Echo => {
                    let count = self.echo;
                    mk_subgraph(
                        count,
                        *module,
                        echo::Echo::get_input_names().into_iter(),
                        echo::Echo::get_output_names().into_iter(),
                        &mut graph.cluster(),
                    );
                    // self.base_graph.echo.push(g.clone());
                    // self.graphs.push(Stmt::Subgraph(g));

                    self.echo += 1;
                    // (module, self.echo)
                }
                ModuleType::Delay => {
                    let count = self.delay;
                    mk_subgraph(
                        count,
                        *module,
                        delay::Delay::get_input_names().into_iter(),
                        delay::Delay::get_output_names().into_iter(),
                        &mut graph.cluster(),
                    );
                    // self.base_graph.delay.push(g.clone());
                    // self.graphs.push(Stmt::Subgraph(g));

                    self.delay += 1;
                    // (module, self.delay)
                }
                ModuleType::Output => {
                    let count = self.output;
                    mk_subgraph(
                        count,
                        *module,
                        output::Output::get_input_names().into_iter(),
                        output::Output::get_output_names().into_iter(),
                        &mut graph.cluster(),
                    );
                    // self.base_graph.output.push(g.clone());
                    // self.graphs.push(Stmt::Subgraph(g));

                    self.output += 1;
                    // (module, self.output)
                }
                ModuleType::Chorus => {
                    let count = self.chorus;
                    mk_subgraph(
                        count,
                        *module,
                        chorus::Chorus::get_input_names().into_iter(),
                        chorus::Chorus::get_output_names().into_iter(),
                        &mut graph.cluster(),
                    );

                    // self.base_graph.chorus.push(g.clone());
                    // self.graphs.push(Stmt::Subgraph(g));

                    self.chorus += 1;
                    // (module, self.chorus)
                }
                ModuleType::Reverb => {
                    let count = self.reverb;
                    mk_subgraph(
                        count,
                        *module,
                        reverb::ReverbModule::get_input_names().into_iter(),
                        reverb::ReverbModule::get_output_names().into_iter(),
                        &mut graph.cluster(),
                    );
                    // self.base_graph.reverb.push(g.clone());
                    // self.graphs.push(Stmt::Subgraph(g));

                    self.reverb += 1;
                    // (module, self.reverb)
                }
                ModuleType::EnvFilter => {
                    let count = self.env;
                    mk_subgraph(
                        count,
                        *module,
                        envelope::EnvelopeFilter::get_input_names().into_iter(),
                        envelope::EnvelopeFilter::get_output_names().into_iter(),
                        &mut graph.cluster(),
                    );
                    // self.base_graph.env.push(g.clone());
                    // self.graphs.push(Stmt::Subgraph(g));

                    self.env += 1;
                    // (module, self.env)
                }
                ModuleType::OverDrive => {
                    let count = self.od;
                    mk_subgraph(
                        count,
                        *module,
                        overdrive::OverDrive::get_input_names().into_iter(),
                        overdrive::OverDrive::get_output_names().into_iter(),
                        &mut graph.cluster(),
                    );
                    // self.base_graph.od.push(g.clone());
                    // self.graphs.push(Stmt::Subgraph(g));

                    self.od += 1;
                    // (module, self.od)
                }
            }
        }
    }
}

// type ModuleSubGraph = Subgraph;
//
// #[derive(Debug, Default, Clone)]
// pub struct ModuleBaseGraph {
//     pub vco: Vec<ModuleSubGraph>,
//     pub lfo: Vec<ModuleSubGraph>,
//     pub mco: Vec<ModuleSubGraph>,
//     pub echo: Vec<ModuleSubGraph>,
//     pub delay: Vec<ModuleSubGraph>,
//     pub chorus: Vec<ModuleSubGraph>,
//     pub reverb: Vec<ModuleSubGraph>,
//     pub env: Vec<ModuleSubGraph>,
//     pub od: Vec<ModuleSubGraph>,
//     pub output: Vec<ModuleSubGraph>,
// }

fn mk_subgraph(
    count: Counter,
    module_name: ModuleType,
    get_inputs: impl IntoIterator<Item = impl Display>,
    get_outputs: impl IntoIterator<Item = impl Display>,
    writer: &mut dot_writer::Scope<'_, '_>,
) {
    // mk circle for each input
    // let mut inputs: Vec<Stmt> = get_inputs.into_iter().enumerate()
    //                 .map(|(output, name)| {
    //                     Stmt::Node(
    //                         node!(&format!("{module_name:?}-{count}-{output}"); attr!("shape", "circle"), attr!("label", &name)),
    //                     )
    //                 })
    //                 .collect();
    // write
    // mk circle for each output
    // let mut outputs: Vec<Stmt> = get_outputs.into_iter().enumerate()
    //                 .map(|(output, name)| {
    //                     Stmt::Node(
    //                         node!(&format!("{module_name:?}-{count}-{output}"); attr!("shape", "circle"), attr!("label", &name)),
    //                     )
    //                 })
    //                 .collect();
    // inputs.append(&mut vec![
    //     Stmt::Attribute(attr!("shape", "square")),
    //     Stmt::Attribute(attr!("label", "inputs")),
    // ]);
    // outputs.append(&mut vec![
    //     Stmt::Attribute(attr!("shape", "square")),
    //     Stmt::Attribute(attr!("label", "outputs")),
    // ]);
    //
    // let input = subgraph!(&format!("{module_name:?}-{count}-inputs"), inputs);
    // let output = subgraph!(&format!("{module_name:?}-{count}-outputs"), outputs);
    //
    // subgraph!(&format!("{module_name:?}-{count}"); input, output, attr!("label", &format!("{module_name:?}")), attr!("shape", "square"))
    // base_graph.vco.push(ModuleSubGraph { inputs, outputs });
    let module_graph = writer
        .set_shape(Shape::Rectangle)
        .set_label(&format!("{module_name:?}"));

    {
        let mut input_box = module_graph.subgraph();
        input_box.set_shape(Shape::Rectangle).set_label("inputs");

        for (input, name) in get_inputs.into_iter().enumerate() {
            input_box
                .node_named(&format!("{module_name:?}-{count}-{input}"))
                .set_shape(Shape::Circle)
                .set_label(&format!("{name}"));
        }
    }

    {
        let mut output_box = module_graph.subgraph();
        output_box.set_shape(Shape::Rectangle).set_label("outputs");

        for (output, name) in get_outputs.into_iter().enumerate() {
            output_box
                .node_named(&format!("{module_name:?}-{count}-{output}"))
                .set_shape(Shape::Circle)
                .set_label(&format!("{name}"));
        }
    }
}

pub fn mk_graph(modules: &[ModuleType]) -> Vec<u8> {
    let mut mods = ModCounter::new(modules);

    mods.mk_graph();

    // while mods.().is_some() {}

    mods.graph
}
