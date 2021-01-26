use std::sync::{ Arc }; 

use druid::widget::prelude::*;
use druid::{
    Lens, WidgetExt, Selector, AppDelegate, Handled, DelegateCtx, Target, Command, Color,  WidgetPod, MouseButton, TimerToken,
    widget::{Flex, Svg, SvgData, Label},
};

const WIDGET_SPACING: f64 = 1.0;
const CELL_SIZE: f64 = 20.0;
pub const TICK_EVENT: Selector<()> = Selector::new("com.dri.event.tick");
pub const PLACE_FUEL_EVENT: Selector<(usize, usize)> = Selector::new("com.dri.event.place-fuel");

pub struct Delegate;

impl AppDelegate<AppState> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut AppState,
        _env: &Env,
    ) -> Handled {
        if cmd.is(TICK_EVENT) {
            for row in 0..data.grid.height() {
                for col in 0..data.grid.width() {
                    let mut cell = data.grid[(row, col)].clone();
                    if cell.of != FuelType::None {
                        if cell.hp > 0 { 
                            cell.hp = std::cmp::max(cell.hp - *data.ticks as u32, 0);
                            data.balance += cell.of.generated_energy();
                        }
                        if cell.hp == 0 {
                            cell.of = FuelType::None;
                        }
                        if !cell.same(&data.grid[(row, col)]) {
                            data.grid[(row, col)] = cell;
                        }
                    }
                }
            }
            return Handled::Yes
        }
        if cmd.is(PLACE_FUEL_EVENT) {
            let index = *cmd.get(PLACE_FUEL_EVENT).expect("place event payload");
            if data.balance >= FuelType::Wood.get_price() {
                data.balance -= FuelType::Wood.get_price();
                data.grid[index] = Fuel::new(
                    FuelType::Wood,
                    FuelType::Wood.max_hp(),
                );
            }
            return Handled::Yes;
        }
        Handled::No
    }
}

#[derive(Clone, Data, PartialEq, Debug)]
pub enum FuelType {
    None,
    Wood,
}

impl FuelType {
    fn max_hp(&self) -> u32 {
        match self {
            FuelType::None => 0,
            FuelType::Wood => 100,
        }
    }
    fn get_svg_data(&self) -> SvgData {
        match self {
            Self::None => include_str!("../assets/svg/empty.svg"),
            Self::Wood => include_str!("../assets/svg/log-wood.svg"),
        }.parse().unwrap()
    }
    fn generated_energy(&self) -> f64 {
        match self {
            Self::None => 0.0,
            Self::Wood => 10.0,
        }
    }
    fn get_price(&self) -> f64 {
        match self {
            Self::None => 0.0,
            Self::Wood => 80.0,
        }
    }
}

#[derive(Clone, Data, Lens, PartialEq, Debug)]
pub struct Fuel {
    pub of: FuelType,
    pub hp: u32,
}

impl Fuel {
    pub fn new(of: FuelType, hp: u32) -> Fuel {
        Fuel { of, hp }
    }
}

impl Default for Fuel {
    fn default() -> Self {
        Self {
            of: FuelType::None,
            hp: 0,
        }
    }
}

#[derive(Clone, Data)]
pub struct Grid {
    inner: Arc<Vec<Fuel>>,
    width: usize,
    height: usize,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Grid {
        Grid {
            inner: Arc::new(vec![
                Fuel::default();
                width * height
            ]),
            width,
            height,
        }
    }
    pub fn height(&self) -> usize { self.height }
    pub fn width(&self) -> usize { self.width }
}

pub struct GridIndex {
    pub row: usize,
    pub col: usize,
}

impl From<(usize, usize)> for GridIndex {
    fn from(index: (usize, usize)) -> GridIndex {
        GridIndex {
            row: index.0,
            col: index.1,
        }
    }
}

impl<T> std::ops::Index<T> for Grid
    where T: Into<GridIndex>
{
    type Output = Fuel;

    fn index(&self, index: T) -> &Self::Output {
        let index = index.into();
        &self.inner[self.width * index.row + index.col]
    }
}

impl<T> std::ops::IndexMut<T> for Grid
    where T: Into<GridIndex>
{
    fn index_mut(&mut self, index: T) -> &mut Self::Output {
        let index = index.into();
        &mut Arc::make_mut(&mut self.inner)[self.width * index.row + index.col]
    }
}

#[derive(Clone, Data, Lens)]
pub struct AppState {
    pub grid: Grid,
    pub balance: f64,
    pub ticks: Arc<f64>,
}

pub struct GridWidget<T> {
    flex: Option<WidgetPod<T, Flex<T>>>,
}

impl<T: Data> GridWidget<T> {
    pub fn new() -> Self {
        GridWidget { flex: None }
    }
}

impl Widget<Grid> for GridWidget<Grid> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut Grid, env: &Env) {
        if let Some(flex) = self.flex.as_mut() {
            flex.event(ctx, event, data, env);
        }
    }
    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &Grid, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            let mut flex = Flex::row();
            for row in 0..data.height() {
                let mut flex_col = Flex::column();
                for col in 0..data.width() {
                    let cell = GridCellWidget::new(row, col)
                        .lens(GridLens { row, col })
                        .fix_size(CELL_SIZE, CELL_SIZE)
                        .border(Color::YELLOW, 3.)
                        .background(Color::WHITE);
                    flex_col.add_child(cell);
                    flex_col.add_spacer(WIDGET_SPACING);
                }
                flex.add_child(flex_col);
                flex.add_spacer(WIDGET_SPACING);
            }
            self.flex = Some(WidgetPod::new(flex));
        }
        if let Some(flex) = self.flex.as_mut() {
            flex.lifecycle(ctx, event, data, env);
        }
    }
    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &Grid, data: &Grid, env: &Env) {
        if let Some(flex) = self.flex.as_mut() {
            flex.update(ctx, data, env);
        }
    }
    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &Grid, env: &Env) -> Size {
        if let Some(flex) = self.flex.as_mut() {
            flex.layout(ctx, bc, data, env)
        } else {
            bc.max()
        }

    }
    fn paint(&mut self, ctx: &mut PaintCtx, data: &Grid, env: &Env) {
        let start = std::time::Instant::now();
        if let Some(flex) = self.flex.as_mut() {
            flex.paint(ctx, data, env);
        }
        println!("paint time: {:?}", std::time::Instant::now() - start);
    }
}

pub struct GridLens {
    row: usize,
    col: usize,
}

impl From<&GridLens> for GridIndex {
    fn from(lens: &GridLens) -> GridIndex {
        GridIndex {
            row: lens.row,
            col: lens.col,
        }
    }
}

impl Lens<Grid, Fuel> for GridLens {
    fn with<V, F: FnOnce(&Fuel) -> V>(&self, state: &Grid, f: F) -> V {
        f(&state[self])
    }
    fn with_mut<V, F: FnOnce(&mut Fuel) -> V>(&self, state: &mut Grid, f: F) -> V {
        let mut fuel = state[self].clone();
        let result = f(&mut fuel);
        if !fuel.same(&state[self]) {
            state[self] = fuel;
        }
        result
    }
}

pub struct GridCellWidget<T> {
    svg: Option<WidgetPod<T, Svg>>,
    pos: (usize, usize),
}

impl<T: Data> GridCellWidget<T> {
    pub fn new(row: usize, col: usize) -> Self {
        GridCellWidget { 
            svg: None,
            pos: (row, col),
        }
    }
}

impl Widget<Fuel> for GridCellWidget<Fuel> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, _data: &mut Fuel, _env: &Env) {
        match event {
            Event::MouseDown(mouse_event) => {
                if mouse_event.button == MouseButton::Left {
                    ctx.set_active(true);
                    ctx.request_paint();
                }
            }
            Event::MouseUp(mouse_event) => {
                if ctx.is_active() && mouse_event.button == MouseButton::Left {
                    ctx.set_active(false);
                    if ctx.is_hot() {
                        ctx.submit_command(Command::new(PLACE_FUEL_EVENT, self.pos, Target::Auto));
                    }
                    ctx.request_paint();
                }
            }
            _ => {}
        }
    }
    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &Fuel, env: &Env) { 
        if let LifeCycle::WidgetAdded = event {
            self.svg = Some(WidgetPod::new(Svg::new(data.of.get_svg_data())));
        }
        if let Some(svg) = self.svg.as_mut() {
            svg.lifecycle(ctx, event, data, env);
        }
    }
    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &Fuel, data: &Fuel, _env: &Env) {
        if !old_data.of.same(&data.of) {
            self.svg = Some(WidgetPod::new(Svg::new(data.of.get_svg_data())));
            ctx.children_changed();
        } 
    }
    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &Fuel, env: &Env) -> Size { 
        if let Some(svg) = self.svg.as_mut() {
            svg.layout(ctx, bc, data, env)
        } else {
            bc.max()
        }
    }
    fn paint(&mut self, ctx: &mut PaintCtx, data: &Fuel, env: &Env) { 
        if let Some(svg) = self.svg.as_mut() {
            svg.paint(ctx, data, env);
        }
    }
}

struct TimerWidget {
    timer_id: TimerToken,
}

impl Widget<AppState> for TimerWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppState, _env: &Env) {
        if let Event::Timer(id) = event {
            if *id == self.timer_id {
                ctx.submit_command(TICK_EVENT);
                let millis = (1000. / *data.ticks) as _;
                let duration = std::time::Duration::from_millis(millis);
                self.timer_id = ctx.request_timer(duration);
            }
        }
    }
    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &AppState, _env: &Env) { 
        if let LifeCycle::WidgetAdded = event {
                let millis = (1000. / *data.ticks) as _;
                let duration = std::time::Duration::from_millis(millis);
                self.timer_id = ctx.request_timer(duration);
        }
    }
    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &AppState, _data: &AppState, _env: &Env) { }
    fn layout(&mut self, _ctx: &mut LayoutCtx, _bc: &BoxConstraints, _data: &AppState, _env: &Env) -> Size {
        Size::ZERO
    }
    fn paint(&mut self, _ctx: &mut PaintCtx, _data: &AppState, _env: &Env) { }
}

pub fn build_root_widget() -> impl Widget<AppState> {
    let balance = Label::new(|data: &f64, _: &Env| format!("Balance: {}", data))
        .lens(AppState::balance);
    Flex::row()
        .with_child(GridWidget::new().lens(AppState::grid))
        .with_child(balance)
        .with_child(TimerWidget { timer_id: TimerToken::INVALID })
}