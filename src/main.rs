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
    gift_chance:GiftChance,
    wonderful_count:usize,
    chapter:Chapter,
    bounty_view:bool
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
    blessing:Vec<AverageRank>,
    burden:Vec<AverageRank>,
    bounty:Vec<AverageRank>
}

type AverageRank = [f64;3];

struct Probabilities {
    chosen:[f64;3],
    rank_up:[f64;2]
}

#[derive(Copy,Clone,Debug)]
enum GType {
    Power,
    Bonus,
    Quick,
    Blessing,
    Burden,
    Bounty
}

#[derive(Copy, Clone, Debug, Default,PartialEq)]
enum Chapter {
    #[default]
    Story,
    AStory,
    Towers,
    SpecialTowers
}

const POWER_COLORS:[&str;3] = ["#2a2a59","#44338e","#9b73d6"];
const BONUS_COLORS:[&str;3] = ["#f0892b","#eabd57","#e3dc66"];
const QUICK_COLORS:[&str;3] = ["#069c80","#0ebc80","#71e380"];
const BLESS_COLORS:[&str;3] = ["#8534ae","#cf2be2","#f648e3"];
const BURDN_COLORS:[&str;3] = ["#8f3937","#c14552","#f03762"];
const BOUNT_COLORS:[&str;3] = ["#527ea8","#66b5d5","#77e1ec"];


fn gift_color(gift_type:GType, rank:usize)-> Color32 {
    let hex_color = match gift_type {
        GType::Power => POWER_COLORS[rank],
        GType::Bonus => BONUS_COLORS[rank],
        GType::Quick => QUICK_COLORS[rank],
        GType::Blessing => BLESS_COLORS[rank],
        GType::Burden => BURDN_COLORS[rank],
        GType::Bounty => BOUNT_COLORS[rank]
    };
    egui::ecolor::Color32::from_hex(hex_color).unwrap()
}

// ------------------- GIFT SPECIFIC FUNCTIONS -----------------

fn gift_probabilities(karma:f64, gift_type:GType, chapter:Chapter) -> Probabilities {
    match gift_type {
        GType::Power => power_probabilities(karma, chapter),
        GType::Bonus => bonus_probabilities(karma, chapter),
        GType::Quick => quick_probabilities(karma, chapter),
        GType::Bounty => bounty_probabilities(karma, chapter),
        GType::Blessing => Probabilities { 
            chosen: power_probabilities(karma,chapter).chosen, 
            rank_up: quick_probabilities(karma,chapter).rank_up },
        GType::Burden => panic!("Burdens do not have any probabilities")
    }
}


fn power_probabilities(karma: f64,chapter:Chapter) -> Probabilities {
    Probabilities {
        chosen: match chapter {
            Chapter::Story => [
                1.0,
                clamp(karma * 0.6, 0.1, 0.9),
                clamp(karma - 0.9, 0.0, 0.9)
            ],
            Chapter::AStory => [0.0;3],
            Chapter::Towers | Chapter::SpecialTowers => [
                1.0,
                clamp(0.2*karma, 0.0, 0.5),
                0.0
            ]
        },
        rank_up: [
            clamp(0.12 + karma*0.5,0.12,0.8),
            clamp(-0.04 + karma*0.4, 0.0, 0.45)
        ]
    }
} 

fn bonus_probabilities(karma: f64, chapter:Chapter) -> Probabilities {
    match chapter {
        Chapter::Story | Chapter::Towers | Chapter::SpecialTowers => Probabilities { 
            chosen: [
                clamp(0.1 + 0.7*karma, 0.25, 0.9),
                clamp(0.7*karma, 0.1, 0.9),
                0.0
            ],
            rank_up: [
                clamp(0.10 + karma*0.7, 0.15, 0.9),
                clamp(-0.06 + karma*0.6, 0.0, 0.5)
            ]},
        Chapter::AStory => Probabilities { 
            chosen: [clamp(0.05 + 0.3*karma, 0.05, 0.9),0.0,0.0], 
            rank_up: [
                clamp(0.1 + 0.3 *karma, 0.15, 0.9),
                clamp(-0.06 + 0.3*karma, 0.0, 0.5)
            ] }
    }
    
    
}

fn quick_probabilities(karma: f64, chapter:Chapter) -> Probabilities {
    match chapter {
        Chapter::Story | Chapter::Towers | Chapter::SpecialTowers => Probabilities { 
            chosen: [
                clamp(0.1 + 0.3*karma, 0.15, 0.5),
                clamp(0.05 + 0.3*karma, 0.05, 0.5),
                0.0
            ],
            rank_up: [
                clamp(0.1 + 0.6*karma, 0.15, 0.8),
                match chapter {
                    Chapter::Story  => clamp(-0.06 + 0.5*karma, 0.0, 0.5),
                    _               => clamp(-0.06 + 0.4*karma, 0.0, 0.5)
                }
            ]},
        Chapter::AStory => Probabilities { 
            chosen: [clamp(0.05 + 0.3*karma, 0.05, 0.9),0.0,0.0], 
            rank_up: [
                clamp(0.1 + 0.3*karma, 0.15, 0.8),
                clamp(-0.06 + 0.3*karma, 0.0, 0.5)
            ] }
    }
    
    
}

fn bounty_probabilities(karma:f64, chapter:Chapter) -> Probabilities {
    match chapter {
        Chapter::Story => Probabilities { 
            chosen: [clamp(-0.02 + 0.2*karma, 0.0, 0.3), 0.0, 0.0], 
            rank_up: [
                clamp(0.1 + 0.5*karma, 0.1, 0.8),
                clamp(-0.05 + 0.5*karma, 0.0, 0.5)
            ] },
        Chapter::AStory => Probabilities { 
            chosen: [clamp(0.1 + 0.5*(karma - 1.0), 0.2, 0.8),0.0, 0.0], 
            rank_up: [
                clamp(0.2 + 0.5*(karma - 1.0), 0.2, 0.8),
                clamp(0.1 + 0.4*(karma - 1.0), 0.1, 0.5)
            ] },
        Chapter::Towers | Chapter::SpecialTowers => Probabilities { 
            chosen: [clamp(0.3*karma, 0.0, 0.5),0.0, 0.0], 
            rank_up: [
                clamp(0.1 + 0.4*karma, 0.1, 0.8),
                clamp(-0.05 + 0.3*karma, 0.0, 0.5)
            ] },
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

fn powhalf(x:usize) -> f64 {
    if x > 0 {
        0.5*powhalf(x - 1)
    } else {
        1.0
    }
}

fn bounty_average_rank(karma:f64, chapter:Chapter) -> AverageRank {
    let mut result = [0.0, 0.0, 0.0];
    let prob = gift_probabilities(karma, GType::Bounty, chapter);
    result[0] = prob.chosen[0];
    result[1] = result[0] * prob.rank_up[0];
    result[2] = result[1] * prob.rank_up[1];
    return result;
}

fn tower_initial_sequence(karma:f64) -> ([f64;3], [AverageRank;6]) {
    let blessing_prob = gift_probabilities(karma, GType::Blessing, Chapter::Towers);
    let two_gift_chance = blessing_prob.chosen[1];

    //blessings
    let mut blessing_ranks = [1.0, blessing_prob.rank_up[0], 0.0];
    blessing_ranks[2] = blessing_ranks[1] * blessing_prob.rank_up[1];

    //assume only one gift is found
    let mut one_gift = [[0.0;3];6];
    {
        let blessing_rank2 = blessing_ranks[1] - blessing_ranks[2];

        //burdens have a 25% chance to have a rank one lower than the blessing
        let mut burden_ranks = [1.0, 0.0, 0.0];
        burden_ranks[1] = blessing_ranks[1] - blessing_rank2 * 0.25;
        burden_ranks[2] = blessing_ranks[2] * 0.75;

        one_gift[GType::Blessing as usize] = blessing_ranks;
        one_gift[GType::Burden as usize] = burden_ranks;
    }

    //assume two gifts are found
    let mut two_gifts = [[0.0;3];6];
    {
        // burdens are 3 star unless none of the two blessings are 3 stars (then they will be 2 stars)
        let mut burden_ranks = [1.0;3];
        burden_ranks[2] = 1.0 - (1.0 - blessing_ranks[2])*(1.0 - blessing_ranks[2]);

        two_gifts[GType::Blessing as usize] = blessing_ranks.map(|r| r*2.0);
        two_gifts[GType::Burden as usize] = burden_ranks;
        
    }

    let gift_ranks = merge(two_gifts, one_gift, two_gift_chance);
    let remaining = [0.0, 0.0, 1.0 - two_gift_chance];
    return (remaining, gift_ranks)
}

fn special_tower_initial_sequence(karma:f64, wonderful_count:usize) -> ([f64;3], [AverageRank;6]) {
    let mut gift_ranks = [[0.0;3];6];
    let power_prob = gift_probabilities(karma, GType::Power, Chapter::Towers);

    //blessings
    let mut power_ranks = [1.0, power_prob.rank_up[0], 0.0];
    power_ranks[2] = power_ranks[1] * power_prob.rank_up[1] * powhalf(wonderful_count);

    //burden split
    let mut burden_ranks = [0.0;3];
    burden_ranks[0] = power_ranks[1]/3.0;
    burden_ranks[1] = power_ranks[2]/2.0;
    burden_ranks[2] = burden_ranks[1];

    //burden merge
    burden_ranks[1] += burden_ranks[2];
    burden_ranks[0] += burden_ranks[1];

    //add results to the general list
    gift_ranks[GType::Power as usize] = power_ranks;
    gift_ranks[GType::Burden as usize] = burden_ranks;

    let remaining = [0.0, 1.0 - burden_ranks[0], 1.0];
    return (remaining, gift_ranks);
}

/// Calculates the average number of gifts of each type after several "Try add gift" actions in a specific order
/// 
fn try_gift_sequence(karma:f64, order:&[GType], wonderful_count:usize, chapter:Chapter) -> [AverageRank;6] {
    //data per gift
    let mut frequency = [0;6];

    //towers and special towers remove a set amount of gifts at the start
    let (mut remaining, mut result) = match chapter {
        Chapter::Towers => tower_initial_sequence(karma),
        Chapter::SpecialTowers => special_tower_initial_sequence(karma, wonderful_count),
        _ => ([1.0; 3], [[0.0; 3]; 6])
    };

    //first power gift counts for blessings
    if let Chapter::SpecialTowers = chapter {frequency[3] += 1}


    //for wonderful gifts: the probability that you have i wonderful gifts
    let mut w_counts = [0.0,0.0,0.0];

    for gift_elem in order {
        let gift_index = *gift_elem as usize;
        let gift_freq = frequency[gift_index];
        if gift_freq >= 3 {
            panic!("Gift {gift_elem:?} occurred in the order list more than 3 times!\nlist:{order:?}")
        }
        let prob = gift_probabilities(karma, *gift_elem, chapter);
        let gift_chance = prob.chosen[gift_freq];
        let gifts_added = apply_probability(&mut remaining, gift_chance);

        result[gift_index][0] += gifts_added;

        //different rank calculations using gifts_added
        let rank2added = gifts_added * prob.rank_up[0];
        result[gift_index][1] +=  rank2added;
        let rank3added = gifts_added * prob.rank_up[1];
        result[gift_index][2] +=  rank3added;

        if let GType::Power = gift_elem {

            // we add the probability that we are in a situation where we have i gifts
            // times the probability that we get a wonderful gift in that situation
            // and add that to the probability we will receive i + 1 wonderful power gifts
            let mut acc_prob = 1.0;
            let rank3chance = rank2added * prob.rank_up[1];
            for i in 0..=2 {
                let situation_chance = (1.0 - w_counts[i]) * acc_prob;
                let added_prob = situation_chance * rank3chance * powhalf(wonderful_count + i);
                acc_prob -= 1.0 - w_counts[i];
                w_counts[i] += added_prob;
            }

            //save the accumulated amount of wonderful gifts
            result[0][2] = w_counts[0] + w_counts[1] + w_counts[2];
        }

        frequency[gift_index] += 1;
    }

    //bounty gifts
    result[5] = bounty_average_rank(karma, chapter);


    return result;
}
/// Creates a new list or averageranks by summing the elements of two lists.
/// The factor argument is the factor of the first list, while 1.0 -factor1 is the factor of the second list
/// Hence if you choose factor 0.5 the resulting list will be the average of the two lists
fn merge(rankings1:[AverageRank;6], rankings2:[AverageRank;6], factor1:f64) -> [AverageRank;6] {
    let mut result = [[0.0;3];6];
    for i in 0..=5 {
        for j in 0..=2 {
            result[i][j] = factor1* rankings1[i][j] + (1.0 - factor1)*rankings2[i][j];
        }
    }
    result
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

    fn story_sequence(&self, i:f64) -> [AverageRank;6] {
        let order1 = [GType::Power, GType::Power, GType::Power, GType::Bonus, GType::Bonus, GType::Quick, GType::Quick];
        let order2 = [GType::Power, GType::Power, GType::Power, GType::Bonus, GType::Quick, GType::Bonus, GType::Quick];
        return merge(
            try_gift_sequence(i, &order1, self.wonderful_count, self.chapter), 
            try_gift_sequence(i, &order2, self.wonderful_count, self.chapter),
            0.5
        );
    }

    fn alter_story_sequence(&self, i:f64) -> [AverageRank;6] {
        let order = [GType::Bonus, GType::Quick];
        try_gift_sequence(i, &order, 0, self.chapter)
    }

    fn tower_sequence(&self, i:f64) -> [AverageRank;6] {
        let order1 = [GType::Bonus, GType::Quick];
        let order2 = [GType::Quick, GType::Bonus];
        return merge(
            try_gift_sequence(i, &order1, 0, self.chapter),
            try_gift_sequence(i, &order2, 0, self.chapter),
            0.5
        );        
    }

    fn special_tower_sequence(&self, i:f64) -> [AverageRank;6] {
        let order1 = [GType::Blessing, GType::Bonus, GType::Bonus];
        let order2 = [GType::Bonus, GType::Bonus, GType::Blessing];
        return merge(
            try_gift_sequence(i, &order1, self.wonderful_count, self.chapter), 
            try_gift_sequence(i, &order2, self.wonderful_count, self.chapter),
            0.5
        );
    }

    fn recalc_giftchance(&mut self) {
        let karma_range = &self.karma_range;
        let mut power = Vec::new();
        let mut bonus = Vec::new();
        let mut quick = Vec::new();
        let mut bounty = Vec::new();
        let mut blessing = Vec::new();
        let mut burden = Vec::new();
        for i in karma_range {
            let i = *i as f64 / 100.0;
            let [power_elem, bonus_elem, quick_elem, bless_elem, burden_elem, bounty_elem] 
                = match self.chapter {
                    Chapter::Story => self.story_sequence(i),
                    Chapter::AStory => self.alter_story_sequence(i),
                    Chapter::Towers => self.tower_sequence(i),
                    Chapter::SpecialTowers => self.special_tower_sequence(i)
            };

            power.push(power_elem);
            bonus.push(bonus_elem);
            quick.push(quick_elem);
            blessing.push(bless_elem);
            burden.push(burden_elem);
            bounty.push(bounty_elem);
        }
        self.gift_chance = GiftChance {power, bonus, quick, blessing, burden, bounty};
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
            let mut recalc = false;
            ui.horizontal(|ui| {
                egui::ComboBox::from_label("Chapter")
                    .selected_text(format!("{:?}",self.chapter))
                    .show_ui(ui, |ui| {
                        if ui.selectable_value(&mut self.chapter, Chapter::Story, "Story").clicked() {recalc = true};
                        if ui.selectable_value(&mut self.chapter, Chapter::AStory, "Alter Story").clicked() {recalc = true};
                        if ui.selectable_value(&mut self.chapter, Chapter::Towers, "Towers").clicked() {recalc = true};
                        if ui.selectable_value(&mut self.chapter, Chapter::SpecialTowers, "Special Towers").clicked() {recalc = true};
                    });
                ui.label("|");
                if ui.checkbox(&mut self.bounty_view, "View bounty gifts").changed() {recalc = true};
            });
            

            ui.horizontal(|ui| {
                let settings = &mut self.domain_settings;
                
                if ui.add(egui::DragValue::new(&mut settings.min)
                    .clamp_range(0..=settings.max)
                    .speed(1.0)
                    .prefix("min: ")).changed() {
                        recalc = true;
                }
                if ui.add(egui::DragValue::new(&mut settings.max)
                    .clamp_range(settings.min..=300)
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
                if ui.add(egui::DragValue::new(&mut self.wonderful_count)
                    .clamp_range(0..=4)
                    .speed(1.0)
                    .prefix("gifts: ")).changed() {
                        recalc = true
                }
                
                
            });
            if recalc {self.recalc()}

            // find barchart based on vector data
            let power_gifts = self.gift_chart(GType::Power, &self.gift_chance.power,"power");
            let mut blessing_gifts = self.gift_chart(GType::Blessing, &self.gift_chance.blessing,"blessing").map(|c| c.stack_on(&[&power_gifts[0]]));
            let mut burden_gifts = self.gift_chart(GType::Burden, &self.gift_chance.burden,"burden").map(|c| c.stack_on(&[&blessing_gifts[0]]));
            let mut bonus_gifts = self.gift_chart(GType::Bonus, &self.gift_chance.bonus,"bonus").map(|c| c.stack_on(&[&burden_gifts[0]]));
            let quick_gifts = self.gift_chart(GType::Quick, &self.gift_chance.quick,"quick").map(|c| c.stack_on(&[&bonus_gifts[0]]));
            
            let bounty_gifts = self.gift_chart(GType::Bounty, &self.gift_chance.bounty, "bounty");

            //different stack order for special tower mode
            if let Chapter::SpecialTowers = self.chapter {
                burden_gifts = burden_gifts.map(|c| c.stack_on(&[&power_gifts[0]]));
                bonus_gifts = bonus_gifts.map(|c| c.stack_on(&[&burden_gifts[0]]));
                blessing_gifts = blessing_gifts.map(|c| c.stack_on(&[&bonus_gifts[0]]));
            }



            //set visibility of graphs
            let mut chart_list = match self.chapter {
                Chapter::Story => vec![power_gifts, bonus_gifts, quick_gifts],
                Chapter::AStory => vec![bonus_gifts, quick_gifts],
                Chapter::Towers => vec![blessing_gifts, burden_gifts, bonus_gifts, quick_gifts],
                Chapter::SpecialTowers => vec![power_gifts, burden_gifts, blessing_gifts, bonus_gifts]
            };
            if self.bounty_view {chart_list = vec![bounty_gifts]}


            egui_plot::Plot::new("my_plot")
                .view_aspect(2.0)
                .allow_drag(false)
                .allow_scroll(false)
                .allow_zoom(false)
                .x_axis_label("karma")
                .y_axis_label("average amount of gifts")
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
