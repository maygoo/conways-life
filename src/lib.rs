//#[allow(clippy::wildcard_imports)]
use seed::{prelude::*, *};

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    // 50vh and vw would be ideal for it to be responsive
    // default values in px
    let universe_width = 950i32;
    let universe_height = 500i32;
    let default_cell_dim = 50i32;

    let length = (universe_height / default_cell_dim) * (universe_width / default_cell_dim);
    let mut universe = vec![Cell::Dead; length as usize];
    universe[9] = Cell::Live;

    let interval_ms = 100;
    orders.stream(streams::interval(interval_ms, || Msg::Tick));

    Model {
        universe: universe,
        universe_dim: (universe_width, universe_height),
        cell_dim: default_cell_dim,
        show_grid: true,
        show_influence: false,
        cursor: (0, 0),
        paused: true,
        mouse_down: false,
    }
}

struct Model {
    //hovered_cell: Box<Cell>,
    // 2d array in row-major order
    universe: Vec<Cell>,
    universe_dim: Vec2, // w x h
    cell_dim: i32,
    // settings
    show_grid: bool,
    show_influence: bool,
    cursor: Vec2,
    paused: bool,
    mouse_down: bool,
}

type Vec2 = (i32, i32);

#[derive(Copy, Clone, PartialEq, Debug)]
enum Cell {
    Live,
    Dead,
}

impl Cell {
    pub fn toggle(&mut self) {
        if self == &Cell::Live {
            *self = Cell::Dead
        } else {
            *self = Cell::Live
        }
    }

    pub fn is_live(&self) -> bool {
        self == &Cell::Live
    }
}

//#[derive(Copy, Clone)]
enum Msg {
    ClickCell(usize),
    MouseMove(Vec2),
    Tick,
    // settings changes
    ToggleGrid,
    ToggleInfl,
    ChangeRatio(String),
    TogglePause,
    MouseDown(bool),
    InfluenceCell(usize),
}

fn update(msg: Msg, model: &mut Model, _: &mut impl Orders<Msg>) {
    match msg {
        Msg::ClickCell(i) => model.universe[i].toggle(),
        Msg::InfluenceCell(i) => model.universe[i] = Cell::Live,
        Msg::Tick if !model.paused => {
            let cols = (model.universe_dim.0 / model.cell_dim) as usize;
            let rows = (model.universe_dim.1 / model.cell_dim) as usize;

            let mut new_universe = model.universe.clone();
            
            for (i, cell) in model.universe.iter().enumerate() {
                let (x, y) = rm_to_xy(i, cols as usize);
                let mut live_neighbours = 0;

                // check each neighbouring cell including diagonals.
                if y > 0 && model.universe[(y-1) * cols + x].is_live() { // up
                    live_neighbours += 1;
                }
                if y < rows - 1 && model.universe[(y+1) * cols + x].is_live() { // down
                    live_neighbours += 1;
                }
                if x > 0 && model.universe[y * cols + x-1].is_live() { // left
                    live_neighbours += 1;
                }
                if x < cols - 1 && model.universe[y * cols + x+1].is_live() { // right
                    live_neighbours += 1;
                }

                if y > 0 && x > 0 && model.universe[(y-1) * cols + x-1].is_live() { // up-left
                    live_neighbours += 1;
                }
                if y > 0 && x < cols - 1 && model.universe[(y-1) * cols + x+1].is_live() { // up-right
                    live_neighbours += 1;
                }
                if y < rows - 1 && x > 0 && model.universe[(y+1) * cols + x-1].is_live() { // down-left
                    live_neighbours += 1;
                }
                if y < rows - 1 && x < cols - 1 && model.universe[(y+1) * cols + x+1].is_live() { // down-right
                    live_neighbours += 1;
                }

                // implement conway's game of life rules
                if cell.is_live() && (live_neighbours < 2 || live_neighbours > 3) {
                    // dies from over and under population
                    new_universe[i] = Cell::Dead;
                } else if !cell.is_live() && live_neighbours == 3 {
                    // new life!
                    new_universe[i] = Cell::Live;
                }
            }

            model.universe = new_universe;
        }
        Msg::Tick => (),
        // settings
        Msg::MouseMove(pos) => model.cursor = pos,
        Msg::ToggleGrid => if model.show_grid { model.show_grid = false } else { model.show_grid = true },
        Msg::ToggleInfl => if model.show_influence { model.show_influence = false } else { model.show_influence = true },
        Msg::TogglePause => if model.paused { model.paused = false } else { model.paused = true },
        Msg::ChangeRatio(dim) => {
            model.cell_dim = dim.parse().unwrap();

            let cols = model.universe_dim.0 / model.cell_dim;
            let rows = model.universe_dim.1 / model.cell_dim;
            model.universe = vec![Cell::Dead; (rows * cols) as usize];
        },
        Msg::MouseDown(true) => model.mouse_down = true,
        Msg::MouseDown(false) => model.mouse_down = false,
    }
}

/// Transform a single row-major coordiante to an x,y coordinate.
fn rm_to_xy(pos: usize, row_len: usize) -> (usize, usize) {
    let x = pos % row_len;
    let y = pos / row_len;
    (x, y)
}

fn view(model: &Model) -> Node<Msg> {
    div![
        style!{
            St::Display => "flex",
            St::JustifyContent => "center",
            St::AlignItems => "center",
            St::FlexDirection => "column",
            St::Height => "100vh",
            St::Width => "100vw",
        },

        // enable mouse event to track user's cursor
        mouse_ev(Ev::MouseMove, |ev| Msg::MouseMove((ev.client_x(), ev.client_y()))),
        view_influence(model),

        view_title(),

        view_settings(model),

        view_universe(model),
    ]
}

fn view_title() -> Node<Msg> {
    div![
        style!{
            St::Margin => "2.5em",
        },
        div![
            style!{
                St::FontSize => "3.5em",
            },
            "Conway's Game of Life",
        ],
        br!(),
        "Built in Rust using the ",
        a![
            attrs!(At::Href => "https://seed-rs.org/"),
            "Seed",
        ],
        " framework. ",
        a![
            attrs!(At::Href => "#"),
            "View source",
        ],
        ".",
    ]
}

fn view_influence(model: &Model) -> Option<Node<Msg>> {
    let left = format!("{}px", model.cursor.0);
    let top = format!("{}px", model.cursor.1);

    let height = "50px";
    let width = "50px";

    IF!(model.show_influence => {
        div![
            attrs!{
                At::Class => "influence",
                At::Id => "Influence",
            },
            style!{
                St::Position => "fixed",
                St::BorderRadius => "50%",
                St::Transform => "translateX(-50%) translateY(-50%)",
                St::PointerEvents => "none",
                St::Left => left,
                St::Top => top,
                St::MixBlendMode => "difference",
                St::ZIndex => "10000",
                St::Border => "2px solid green",
                St::Height => height,
                St::Width => width,
            },
        ]
    })
}

fn view_settings(model: &Model) -> Node<Msg> {
    div![
        style!{
            St::Margin => "2em",
        },

        div![
            style!{
                St::Display => "flex",
                St::AlignItems => "center",
            },
            "Universe Size: ",
            br!(),
            input![
                attrs!{
                    At::Type => "radio",
                    At::Name => "ratio",
                    At::Value => "5",
                },
                input_ev(Ev::Input, Msg::ChangeRatio),
            ],
            label!("5"),
            input![
                attrs!{
                    At::Type => "radio",
                    At::Name => "ratio",
                    At::Value => "10",
                },
                input_ev(Ev::Input, Msg::ChangeRatio),
            ],
            label!("10"),
            input![
                attrs!{
                    At::Type => "radio",
                    At::Name => "ratio",
                    At::Value => "25",
                },
                input_ev(Ev::Input, Msg::ChangeRatio),
            ],
            label!("25"),
            input![
                attrs!{
                    At::Type => "radio",
                    At::Name => "ratio",
                    At::Value => "50",
                },
                IF!(model.cell_dim == 50 => attrs!(At::Checked => "checked")),
                input_ev(Ev::Input, Msg::ChangeRatio),
            ],
            label!("50"),
        ],
        br!(),
        
        div![
            style!{
                St::Display => "flex",
                St::AlignItems => "center",
            },
            "Click cells while paused.",
            button![
                style!(St::MarginLeft => "1em"),
                if model.paused { "Play" } else { "Pause" },
                ev(Ev::Click, |_| Msg::TogglePause),
            ],
        ],
        br!(),
        
        input![
            attrs!{
                At::Type => "checkbox",
                At::Checked => model.show_grid.as_at_value(),
            },
            ev(Ev::Click, |_| Msg::ToggleGrid),
        ],
        label!("Show grid", ev(Ev::Click, |_| Msg::ToggleGrid)),

        input![
            attrs!{
                At::Type => "checkbox",
                At::Checked => model.show_influence.as_at_value(),
            },
            ev(Ev::Click, |_| Msg::ToggleInfl),
        ],
        label!("Show influence", ev(Ev::Click, |_| Msg::ToggleInfl)),
    ]
}

fn view_universe(model: &Model) -> Node<Msg> {
    let universe_w_px = format!("{}px", model.universe_dim.0);
    let universe_h_px = format!("{}px", model.universe_dim.1);

    let cell_dim_px = format!("{}px", model.cell_dim);

    div![
        style!{
            St::Display => "flex",
            St::JustifyContent => "center",
            St::AlignItems => "center",
            St::UserSelect => "none",
            St::Outline => "solid 1px black",
            St::Height => universe_h_px,
            St::Width => universe_w_px,
            St::UserSelect => "none",
            St::FlexWrap => "wrap",
        },

        model.universe.iter().enumerate().map(|(i, cell)| {
            div![
                style!{
                    St::Outline => if model.show_grid { "solid 1px black" } else { "" }
                    St::BackgroundColor => if cell == &Cell::Live { "black" } else { "white" },
                    St::Height => cell_dim_px,
                    St::Width => cell_dim_px,
                },
                IF!(model.paused => ev(Ev::Click, move |_| Msg::ClickCell(i))),
                IF!(model.paused && model.mouse_down => ev(Ev::MouseEnter, move |_| Msg::InfluenceCell(i))),
                ev(Ev::MouseDown, |_| Msg::MouseDown(true)),
                ev(Ev::MouseUp, |_| Msg::MouseDown(false)),
            ]
        })
    ]
}

#[wasm_bindgen(start)]
pub fn start() {
    App::start("app", init, update, view);
}
