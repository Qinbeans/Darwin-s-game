use std::collections::HashMap;
use ggegui::{egui, Gui};
use ggez::graphics::Color;
use ggez::{Context, graphics};
use ggez::event::EventHandler;
use serde::Deserialize;
use rand::Rng;

const CONFIG_PATH: &str = "./config.json";
const BG_COLOR: (u8, u8, u8) = (0, 0, 0);
const MENU_SIZE: (f32, f32) = (200.0, 200.0);
const SPLIT_FORCE: f32 = -1.0;

#[derive(Debug, Clone, Deserialize)]
pub struct Cells {
    color: Color,
    size: (u32, u32),
    pool: u32,
    speed: f32,
    division: u32,
    rules: HashMap<String, f32>, // Add 'a lifetime bound here
}

#[derive(Debug, Clone, Copy)]
pub struct Cell {
    id: u32,
    cell_type: &'static str,
    color: Color,
    size: (u32, u32),
    x: f32,
    y: f32,
}

impl Cell {
    pub fn new(id: u32, cell_type: &'static str, color: Color, size: (u32, u32), x: f32, y: f32) -> Self {
        Cell {
            id,
            cell_type,
            color,
            size,
            x,
            y,
        }
    }
}

pub struct GoL {
    //...game state
    config: HashMap<&'static str, Cells>, //use map to decrease lookup time
    cells: Vec<Cell>,
    gui: Gui,
    started: bool,
}

impl GoL {
    pub fn new(ctx: &mut Context) -> Self {
        //...initialize game state
        // read in JSON file
        let mut rng = rand::thread_rng();
        // get window size
        ctx.gfx.window().set_fullscreen(Some(ggez::winit::window::Fullscreen::Borderless(None)));
        // prevent escape from exiting game
        let size = ctx.gfx.window().inner_size();

        println!("Window size: {}x{}", size.width, size.height);
        let mut cells = Vec::new();
        let mut cell_id = 0;
        let mut positions = Vec::new();
        let config = std::fs::read_to_string(CONFIG_PATH).expect("Failed to read config file!");
        let config = Box::leak(config.into_boxed_str());
        let config: HashMap<&'static str, Cells> = serde_json::from_str(config).expect("Failed to parse config file!");
        for c in &config {
            for _ in 0..c.1.pool {
                // randomly generate positions
                let mut position = (rng.gen_range(0..size.width as u32) as f32, rng.gen_range(0..size.height as u32) as f32);
                // check if position is already taken
                while positions.contains(&position) {
                    position = (rng.gen_range(0..size.width as u32) as f32, rng.gen_range(0..size.height as u32) as f32);
                }
                positions.push(position);
                let (x, y) = position;
                cells.push(Cell::new(cell_id, c.0, c.1.color, c.1.size, x, y));
                cell_id += 1;
            }
        }
        GoL {
            config,
            cells,
            gui: Gui::new(ctx),
            started: false,
        }
    }
    fn respawn(&mut self, ctx: &mut Context) {
        let mut rng = rand::thread_rng();
        let mut positions = Vec::new();
        let size = ctx.gfx.window().inner_size();
        let mut cell_id = 0;
        // clear cells
        self.cells.clear();
        for c in &self.config {
            for _ in 0..c.1.pool {
                // randomly generate positions
                let mut position = (rng.gen_range(0..size.width as u32) as f32, rng.gen_range(0..size.height as u32) as f32);
                // check if position is already taken
                while positions.contains(&position) {
                    position = (rng.gen_range(0..size.width as u32) as f32, rng.gen_range(0..size.height as u32) as f32);
                }
                positions.push(position);
                let (x, y) = position;
                self.cells.push(Cell::new(cell_id, c.0, c.1.color, c.1.size, x, y));
                cell_id += 1;
            }
        }
    }
}

impl EventHandler for GoL {
    fn quit_event(&mut self, ctx: &mut Context) -> Result<bool, ggez::GameError> {
        if ctx.keyboard.is_key_pressed(ggez::input::keyboard::KeyCode::Escape) {
            return Ok(true);
        }
        Ok(false)
    }
    fn update(&mut self, ctx: &mut Context) -> ggez::GameResult {
        let gui_ctx = self.gui.ctx();
        if self.started {
            //if escape is pressed, set started to false
            if ctx.keyboard.is_key_just_pressed(ggez::input::keyboard::KeyCode::Escape) {
                self.started = false;
            }
            let mut cell_copy = self.cells.clone();
            let len = cell_copy.len();

            let mut splits: Vec<u32> = Vec::new();

            for cell in &mut cell_copy {
                // check size for division
                let cell_type = &self.config[&cell.cell_type];
                if cell.size.0 > cell_type.division && splits.contains(&cell.id){
                    // split cell, the new cell should spawn right next to the original cell and repel away using SPLIT_FORCE
                    let position = (cell.x + cell.size.0 as f32, cell.y + cell.size.1 as f32);
                    let size = (cell.size.0 / 2, cell.size.1 / 2);
                    let new_cell = Cell::new(len as u32, cell.cell_type, cell.color, size, position.0, position.1);
                    self.cells.push(new_cell);
                    cell.size.0 /= 2;
                    cell.size.1 /= 2;
                    let distance = ((cell.x - position.0).powi(2) + (cell.y - position.1).powi(2)).sqrt();
                    let vx = SPLIT_FORCE * (position.0 - cell.x) / distance;
                    let vy = SPLIT_FORCE * (position.1 - cell.y) / distance;
                    cell.x += vx * cell_type.speed;
                    cell.y += vy * cell_type.speed;
                    splits.push(cell.id);
                    splits.push(len as u32)
                }
            }
            for cell in &mut self.cells {
                // find cell type in config
                let cell_type = &self.config[&cell.cell_type];
                let rules = &cell_type.rules;

                if rules.len() == 0 {
                    continue;
                }
                
                for other_cell in &mut cell_copy {
                    if cell.id == other_cell.id {
                        continue;
                    }
                    if other_cell.size.0 == 0 && other_cell.size.1 == 0 {
                        continue;
                    }
                    // Check if cells are overlapping
                    if ((cell.x + cell.size.0 as f32) >= other_cell.x) && (cell.x <= (other_cell.x + other_cell.size.0 as f32)) && ((cell.y + cell.size.1 as f32) >= other_cell.y) && (cell.y <= (other_cell.y + other_cell.size.1 as f32)) {
                        if cell.size.0 > other_cell.size.0 && cell.size.1 > other_cell.size.1 {
                            cell.size.0 += other_cell.size.0;
                            cell.size.1 += other_cell.size.1;
                            other_cell.size.0 = 0;
                            other_cell.size.1 = 0;
                        } else if cell.size.0 < other_cell.size.0 && cell.size.1 < other_cell.size.1 {
                            other_cell.size.0 += cell.size.0;
                            other_cell.size.1 += cell.size.1;
                            cell.size.0 = 0;
                            cell.size.1 = 0;
                        }
                    }

                    if !rules.contains_key(other_cell.cell_type) {
                        continue;
                    }

                    let acceleration = rules[other_cell.cell_type];
                    let (x, y) = (cell.x as f32, cell.y as f32);
                    let (other_x, other_y) = (other_cell.x as f32, other_cell.y as f32);

                    // calculate distance between cells
                    let distance = ((x - other_x).powi(2) + (y - other_y).powi(2)).sqrt();
                    // calculate velocity
                    let vx = acceleration * (other_x - x) / distance;
                    let vy = acceleration * (other_y - y) / distance;

                    cell.x += vx * cell_type.speed;
                    cell.y += vy * cell_type.speed;

                    // if the cell hits the edge of the screen, wrap it around
                    if (cell.x - cell.size.0 as f32) >= ctx.gfx.window().inner_size().width as f32 {
                        cell.x = 0.0;
                    }
                    if (cell.y - cell.size.1 as f32) >= ctx.gfx.window().inner_size().height as f32 {
                        cell.y = 0.0;
                    }
                    if cell.x <= 0.0 - cell.size.0 as f32 {
                        cell.x = ctx.gfx.window().inner_size().width as f32;
                    }
                    if cell.y <= 0.0 - cell.size.1 as f32 {
                        cell.y = ctx.gfx.window().inner_size().height as f32;
                    }
                }
            }
        } else {
            //...handle input
            let size = ctx.gfx.window().inner_size();
            // spawn window in center of screen
            let x = size.width as f32 / 2.0 - MENU_SIZE.0 / 2.0;
            let y = size.height as f32 / 2.0 - MENU_SIZE.1 / 2.0;
            egui::Window::new("Game of Life")
                .default_size(MENU_SIZE)
                .resizable(false)
                .collapsible(false)
                .fixed_pos([x, y])
                .show(&gui_ctx, |ui| {
                    ui.label("Game of Life");
                    if ui.button("Start").clicked() {
                        self.started = true;
                    }
                    if ui.button("Respawn").clicked() {
                        self.respawn(ctx);
                    }
                    if ui.button("Exit").clicked() {
                        std::process::exit(0);
                    }
                    ui.label("Press escape to exit the simulation.");
                    ui.separator();
                    ui.collapsing("Configuration", |ui| {
                        ui.collapsing("Cell Types", |ui| {
                            let mut config_copy = self.config.clone();
                            for cell in &mut config_copy {
                                ui.collapsing(format!("{}", cell.0), |ui| {
                                    // ui.label(format!("Pool: {}", cell.1.pool));
                                    // change to slider
                                    if let Some(cell) = self.config.get_mut(cell.0) {
                                        if ui.add(egui::Slider::new(&mut cell.pool, 0..=1000).text("Pool")).changed() {
                                            // Handle pool change
                                            cell.pool = cell.pool;
                                        }
                                        if ui.add(egui::Slider::new(&mut cell.speed, 0.0..=10.0).text("Speed")).changed() {
                                            // Handle speed change
                                            cell.speed = cell.speed.round();
                                        }
                                        if ui.add(egui::Slider::new(&mut cell.division, 0..=100).text("Division")).changed() {
                                            // Handle division change
                                            cell.division = cell.division;
                                        }
                                    }
                                    ui.collapsing(format!("{} Rules", cell.0), |ui| {
                                        for rule in &mut cell.1.rules {
                                            ui.label(format!("{}: {}", rule.0, rule.1));
                                        }
                                    });
                                    ui.separator();
                                });
                            }
                        });
                    });
                    ui.separator();
                    ui.collapsing("Credits" , |ui| {
                        ui.label("Ryan Fong");
                        ui.add(egui::Hyperlink::from_label_and_url("My Github","https://github.com/qinbeans"));
                        ui.add(egui::Hyperlink::from_label_and_url("My Website","https://qinbeans.github.io"));
                    });
                });
        }
        self.gui.update(ctx);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> ggez::GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::from(BG_COLOR));
        // draw cells
        for cell in &self.cells {
            // let rect = graphics::Rect::new(cell.x as f32, cell.y as f32, cell.size.0 as f32, cell.size.1 as f32);
            let circ = graphics::Mesh::new_circle(ctx, graphics::DrawMode::fill(), [0.0, 0.0], cell.size.0 as f32, 0.1, cell.color)?;
            canvas.draw(&circ, graphics::DrawParam::default().dest([cell.x as f32, cell.y as f32]));
        }
        if !self.started {
            canvas.draw(
                &self.gui,
                graphics::DrawParam::default().dest([0.0, 0.0]),
            )
        }
        //...draw game state
        canvas.finish(ctx)
    }
}