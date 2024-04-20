use eframe::egui;
use egui_plot as plt;

use egui::ecolor::Color32;
fn main() {
    let mut native_options = eframe::NativeOptions::default();
    native_options.viewport = native_options.viewport.with_inner_size([350.0,450.0]);
    let _ = eframe::run_native("Karma Plot", native_options, Box::new(|cc| Box::new(PlotProgram::new(cc))))
                .expect("Unexpected error");
    
}

#[derive(Default,Clone)]
struct PlotProgram { 
    karma_range:Vec<i32>,
    domain_settings:DomainSettings,
    gift_chance:GiftChance
}

#[derive(Default,Clone)]
struct DomainSettings {
    min:i32,
    max:i32,
    step:i32
}

#[derive(Default,Clone)]
struct GiftChance {
    power:Vec<AverageRank>,
    bonus:Vec<AverageRank>,
    quick:Vec<AverageRank>,
    bounty:Vec<AverageRank>
}

type AverageRank = [f64;3];

struct Probabilities {
    chosen:[f64;3],
}

#[derive(Copy,Clone,Debug)]
enum GType {
    Power,
    Bonus,
    Quick
}

enum GRank {
    Simple,
    Lovely,
    Wonderful,
}

const POWER_COLORS:[&str;3] = ["#2a2a59","#44338e","#9b73d6"];
const BONUS_COLORS:[&str;3] = ["#f0892b","#eabd57","#e3dc66"];
const QUICK_COLORS:[&str;3] = ["#069c80","#0ebc80","#71e380"];

fn gift_color(gift_type:GType, rank:usize)-> Color32 {
    let hex_color = match gift_type {
        GType::Power => POWER_COLORS[rank],
        GType::Bonus => BONUS_COLORS[rank],
        GType::Quick => QUICK_COLORS[rank]
    };
    egui::ecolor::Color32::from_hex(hex_color).unwrap()
}

// ------------------- GIFT SPECIFIC FUNCTIONS -----------------

fn gift_probabilities(karma:f64, gift_type:GType) -> Probabilities {
    match gift_type {
        GType::Power => power_probabilities(karma),
        GType::Bonus => bonus_probabilities(karma),
        GType::Quick => quick_probabilities(karma)
    }
}


fn power_probabilities(karma: f64) -> Probabilities {
    Probabilities {
        chosen: [
            1.0,
            clamp(karma * 0.6, 0.1, 0.9),
            clamp(karma - 0.9, 0.0, 0.9)
        ]
    }
} 

fn bonus_probabilities(karma: f64) -> Probabilities {
    Probabilities { 
        chosen: [
            clamp(0.1 + 0.7*karma, 0.25, 0.9),
            clamp(0.7*karma, 0.1, 0.9),
            0.0
        ]
    }
}

fn quick_probabilities(karma: f64) -> Probabilities {
    Probabilities { 
        chosen: [
            clamp(0.1 + 0.3*karma, 0.15, 0.5),
            clamp(0.05 + 0.3*karma, 0.05, 0.5),
            0.0
        ]
    }
}

// -------------- ARITHMATIC FUNCTIONS ---------------

fn clamp(i:f64, min:f64, max:f64) -> f64 {
    if i < min {
        min
    } else if i > max {
        max
    } else {
        i
    }
}

fn apply_probability(remaining:&mut [f64;3], chance:f64) -> f64 {
    let mut total_added = 0.0;
    let mut accumulated_prob = 1.0;
    for i in 0..=2 {
        let current = remaining[i] * chance * accumulated_prob;
        
        //added to later slots if current slot was full
        accumulated_prob *= 1.0 - remaining[i];

        //fill up current slot
        total_added += current;
        remaining[i] -= current;
    }
    total_added
}

/// Calculates the average number of gifts of each type after several "Try add gift" actions in a specific order
/// 
fn try_gift_sequence(karma:f64, order:&[GType]) -> [AverageRank;3] {
    //data per gift
    let mut frequency = [0,0,0];
    let mut result = [[0.0;3];3];

    //open gift slots
    let mut remaining = [1.0, 1.0, 1.0];

    for gift_elem in order {
        let gift_index = *gift_elem as usize;
        let gift_freq = frequency[gift_index];
        if gift_freq >= 3 {
            panic!("Gift {gift_elem:?} occurred in the order list more than 3 times!\nlist:{order:?}")
        }
        let gift_chance = gift_probabilities(karma, *gift_elem).chosen[gift_freq];
        let gifts_added = apply_probability(&mut remaining, gift_chance);

        result[gift_index][0] += gifts_added;

        //different rank calculations using gifts_added

        frequency[gift_index] += 1;
    }
    return result;
}


// ------------ PROGRAM ----------------
impl PlotProgram {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // egui_extras::install_image_loaders(&cc.egui_ctx);
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        let mut program = PlotProgram {
            domain_settings:DomainSettings { min: 0, max: 200, step: 1 },
            ..Self::default()};
        program.recalc();
        program
    }

    fn recalc(&mut self) {
        let settings = &self.domain_settings;
        self.karma_range = (settings.min..=settings.max).step_by(settings.step as usize).collect();
        self.recalc_giftchance();
    }


    fn recalc_giftchance(&mut self) {
        let karma_range = &self.karma_range;
        let mut power = Vec::new();
        let mut bonus = Vec::new();
        let mut quick = Vec::new();
        let mut bounty = Vec::new();
        for i in karma_range {
            let i = *i as f64 / 100.0;
            let mut remaining = [1.0, 1.0, 1.0];

            let mut power_elem = 0.0;
            let power_chance = power_probabilities(i);
            for j in 0..=2 {
                let chance = power_chance.chosen[j];
                power_elem += apply_probability(&mut remaining, chance);
            }

            //bonus
            let mut bonus_elem = 0.0;
            let bonus_c = bonus_probabilities(i).chosen;
            let mut quick_elem = 0.0;
            let quick_c = quick_probabilities(i).chosen;

            //first try
            bonus_elem += apply_probability(&mut remaining, bonus_c[0]);
            //split into two alternative universes: bonus, bonus, quick, quick and bonus, quick, bonus, quick,
            let mut remaining2 = remaining.clone();

            //second try
            quick_elem += 0.5*apply_probability(&mut remaining2, quick_c[0]);
            bonus_elem += 0.5*apply_probability(&mut remaining, bonus_c[1]);

            //third try
            bonus_elem += 0.5*apply_probability(&mut remaining2, bonus_c[1]);
            quick_elem += 0.5*apply_probability(&mut remaining, quick_c[0]);

            //fourth try
            quick_elem += 0.5*apply_probability(&mut remaining2, quick_c[1]);
            quick_elem += 0.5*apply_probability(&mut remaining, quick_c[1]);

            //quick gifts

            let mut bounty_elem = 1.0;

            power.push([power_elem, power_elem/2.0, power_elem/4.0]);
            bonus.push([bonus_elem, bonus_elem/2.0, bonus_elem/4.0]);
            quick.push([quick_elem, quick_elem/2.0, quick_elem/4.0]);
            bounty.push([bounty_elem, bounty_elem/2.0, bounty_elem/4.0]);
            


        }
        self.gift_chance = GiftChance {power,bonus, quick,bounty};
    }

    fn gift_chart(&self, gift_type:GType, average_ranks:&Vec<AverageRank>, name:&str) -> [plt::BarChart;3] {
        let simple_gift =       self.single_rank_chart(gift_color(gift_type,0), 0, &average_ranks, &format!("{name} 1 star"));
        let lovely_gift =       self.single_rank_chart(gift_color(gift_type,1), 1, &average_ranks, &format!("{name} 2 star"));
        let wonderful_gift =    self.single_rank_chart(gift_color(gift_type,2), 2, &average_ranks, &format!("{name} 3 star"));
        [simple_gift, lovely_gift, wonderful_gift]
    }

    fn single_rank_chart(&self, color:Color32,rank:usize, average_ranks:&Vec<AverageRank>, name:&str) -> plt::BarChart {
        plt::BarChart::new(
            self.karma_range.iter().enumerate().map(|(i,karma)| {
                let width = self.domain_settings.step as f64;
                plt::Bar::new(*karma as f64, average_ranks[i][rank])
                    .width(width + 0.2)
                    .fill(color)
        }).collect()).name(name).color(color)
    }

}


// ----------------------- USER INTERACTION -------------------------



impl eframe::App for PlotProgram {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            //setting buttons
            ui.horizontal(|ui| {
                let settings = &mut self.domain_settings;
                let mut recalc = false;
                if ui.add(egui::DragValue::new(&mut settings.min)
                    .clamp_range(0..=settings.max)
                    .speed(1.0)
                    .prefix("min: ")).changed() {
                        recalc = true;
                }
                if ui.add(egui::DragValue::new(&mut settings.max)
                    .clamp_range(settings.min..=200)
                    .speed(1.0)
                    .prefix("max: ")).changed() {
                        recalc = true
                }
                if ui.add(egui::DragValue::new(&mut settings.step)
                    .clamp_range(1..=(settings.max - settings.min))
                    .speed(1.0)
                    .prefix("step: ")).changed() {
                        recalc = true
                }
                if recalc {self.recalc()}
            });
            let power_color = egui::ecolor::Color32::from_hex("#2a3c78").unwrap();
            let bonus_color = egui::ecolor::Color32::from_hex("#e3dc66").unwrap();
            let quick_color = egui::ecolor::Color32::from_hex("#51da6d").unwrap();

            let power_gifts = self.gift_chart(GType::Power,&self.gift_chance.power,"power");
            let bonus_gifts = self.gift_chart(GType::Bonus,&self.gift_chance.bonus,"bonus").map(|c| c.stack_on(&[&power_gifts[0]]));
            let quick_gifts = self.gift_chart(GType::Quick,&self.gift_chance.quick,"quick").map(|c| c.stack_on(&[&bonus_gifts[0]]));


            let chart_list = [power_gifts, bonus_gifts, quick_gifts];
            egui_plot::Plot::new("my_plot")
                .view_aspect(2.0)
                .allow_drag(false)
                .allow_scroll(false)
                .allow_zoom(false)
                .x_axis_label("karma")
                .y_axis_label("average amount")
                .legend(plt::Legend::default().position(plt::Corner::LeftTop))
                .show(ui, |plot_ui| {
                    for list in chart_list {
                        for chart in list {
                            plot_ui.bar_chart(chart);
                        }
                    }
                });
        });
    }
}
