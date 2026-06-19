use crate::{
    app_context::{AppContext, UpdateContext},
    apps::{
        games::{
            dino::{self, DinoState},
            menu::GamesMenuState,
            pong::{self, PongState},
            snake::{self, SnakeState},
            stack::{self, StackState},
            stick_needle::{self, StickNeedleState},
            tetris::{self, TetrisState},
        },
        main_menu::MainMenuState,
        settings::TimeSettingsState,
        time_tools::{
            countdown::{self, CountdownState},
            menu::TimeToolsMenuState,
            pomodoro::{self, PomodoroState},
            stopwatch::{self, StopwatchState},
        },
    },
};

mod clock;
mod games;
mod main_menu;
mod settings;
mod time_tools;

pub enum App {
    Clock,
    MainMenu(MainMenuState),
    TimeToolsMenu(TimeToolsMenuState),
    Stopwatch(StopwatchState),
    Countdown(CountdownState),
    Pomodoro(PomodoroState),

    GamesMenu(GamesMenuState),
    Snake(SnakeState),
    Tetris(TetrisState),
    Pong(PongState),
    Dino(DinoState),
    Stack(StackState),
    StickNeedle(StickNeedleState),

    TimeSettings(TimeSettingsState),
}

impl App {
    pub fn update(&mut self, ctx: &mut UpdateContext) -> Option<App> {
        match self {
            App::MainMenu(state) => main_menu::update(ctx, state),
            App::Clock => clock::update(ctx),
            App::TimeToolsMenu(state) => time_tools::menu::update(ctx, state),
            App::Stopwatch(state) => stopwatch::update(ctx, state),
            App::Countdown(state) => countdown::update(ctx, state),
            App::Pomodoro(state) => pomodoro::update(ctx, state),
            App::GamesMenu(state) => games::menu::update(ctx, state),
            App::Snake(state) => snake::update(ctx, state),
            App::Tetris(state) => tetris::update(ctx, state),
            App::Pong(state) => pong::update(ctx, state),
            App::Dino(state) => dino::update(ctx, state),
            App::Stack(state) => stack::update(ctx, state),
            App::StickNeedle(state) => stick_needle::update(ctx, state),
            App::TimeSettings(state) => settings::update(ctx, state),
        }
    }

    pub fn draw(&self, ctx: &mut AppContext) -> anyhow::Result<()> {
        match self {
            App::MainMenu(state) => main_menu::draw(ctx, state),
            App::Clock => clock::draw(ctx),
            App::TimeToolsMenu(state) => time_tools::menu::draw(ctx, state),
            App::Stopwatch(state) => stopwatch::draw(ctx, state),
            App::Countdown(state) => countdown::draw(ctx, state),
            App::Pomodoro(state) => pomodoro::draw(ctx, state),
            App::GamesMenu(state) => games::menu::draw(ctx, state),
            App::Snake(state) => snake::draw(ctx, state),
            App::Tetris(state) => tetris::draw(ctx, state),
            App::Pong(state) => pong::draw(ctx, state),
            App::Dino(state) => dino::draw(ctx, state),
            App::Stack(state) => stack::draw(ctx, state),
            App::StickNeedle(state) => stick_needle::draw(ctx, state),
            App::TimeSettings(state) => settings::draw(ctx, state),
        }
        Ok(())
    }

    pub fn main_menu() -> Self {
        Self::MainMenu(Default::default())
    }

    pub fn time_tools_menu() -> Self {
        Self::TimeToolsMenu(Default::default())
    }

    pub fn time_settings() -> Self {
        Self::TimeSettings(Default::default())
    }

    pub fn stopwatch() -> Self {
        Self::Stopwatch(Default::default())
    }

    pub fn countdown() -> Self {
        Self::Countdown(Default::default())
    }

    pub fn pomodoro() -> Self {
        Self::Pomodoro(Default::default())
    }

    pub fn games_menu() -> Self {
        Self::GamesMenu(Default::default())
    }

    fn snake_game() -> App {
        Self::Snake(Default::default())
    }

    fn tetris_game() -> App {
        Self::Tetris(Default::default())
    }

    fn pong_game() -> App {
        Self::Pong(Default::default())
    }

    fn dino_game() -> App {
        Self::Dino(Default::default())
    }

    fn stack_game() -> App {
        Self::Stack(Default::default())
    }

    fn stick_needle_game() -> App {
        Self::StickNeedle(Default::default())
    }
}
