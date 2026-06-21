use crate::{
    app_context::{AppContext, UpdateContext},
    apps::{
        games::{
            brick::{self, BrickState},
            dino::{self, DinoState},
            flappy_bird::{self, FlappyBirdState},
            game_2048::{self, Game2048State},
            gold_miner::{self, GoldMinerState},
            gomoku::{self, GomokuState},
            jump_jump::{self, JumpJumpState},
            menu::GamesMenuState,
            naval_battle::{self, NavalBattleState},
            pacman::{self, PacmanState},
            pong::{self, PongState},
            snake::{self, SnakeState},
            stack::{self, StackState},
            stick_needle::{self, StickNeedleState},
            tank_trouble::{self, TankTroubleState},
            target::{self, TargetState},
            tetris::{self, TetrisState},
            undertale::{self, UndertaleState},
            whac_mole::{self, WhacState},
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
    Target(TargetState),
    JumpJump(JumpJumpState),
    WhacMole(WhacState),
    NavalBattle(NavalBattleState),
    TankTrouble(TankTroubleState),
    Game2048(Game2048State),
    FlappyBird(FlappyBirdState),
    GoldMiner(GoldMinerState),
    Gomuku(GomokuState),
    Brick(BrickState),
    Undertale(UndertaleState),
    Pacman(PacmanState),

    TimeSettings(TimeSettingsState),
    SettingsMenu(settings::SettingsMenuState),
    NetworkSettings(settings::NetworkSettingsState),
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
            App::Target(state) => target::update(ctx, state),
            App::JumpJump(state) => jump_jump::update(ctx, state),
            App::WhacMole(state) => whac_mole::update(ctx, state),
            App::NavalBattle(state) => naval_battle::update(ctx, state),
            App::TankTrouble(state) => tank_trouble::update(ctx, state),
            App::Game2048(state) => game_2048::update(ctx, state),
            App::FlappyBird(state) => flappy_bird::update(ctx, state),
            App::GoldMiner(state) => gold_miner::update(ctx, state),
            App::Gomuku(state) => gomoku::update(ctx, state),
            App::Brick(state) => brick::update(ctx, state),
            App::Undertale(state) => undertale::update(ctx, state),
            App::Pacman(state) => pacman::update(ctx, state),
            App::TimeSettings(state) => settings::time::update(ctx, state),
            App::SettingsMenu(state) => settings::menu::update(ctx, state),
            App::NetworkSettings(state) => settings::network::update(ctx, state),
        }
    }

    pub fn draw(&mut self, ctx: &mut AppContext) -> anyhow::Result<()> {
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
            App::Target(state) => target::draw(ctx, state),
            App::JumpJump(state) => jump_jump::draw(ctx, state),
            App::WhacMole(state) => whac_mole::draw(ctx, state),
            App::NavalBattle(state) => naval_battle::draw(ctx, state),
            App::TankTrouble(state) => tank_trouble::draw(ctx, state),
            App::Game2048(state) => game_2048::draw(ctx, state),
            App::FlappyBird(state) => flappy_bird::draw(ctx, state),
            App::GoldMiner(state) => gold_miner::draw(ctx, state),
            App::Gomuku(state) => gomoku::draw(ctx, state),
            App::Brick(state) => brick::draw(ctx, state),
            App::Undertale(state) => undertale::draw(ctx, state),
            App::Pacman(state) => pacman::draw(ctx, state),
            App::TimeSettings(state) => settings::time::draw(ctx, state),
            App::SettingsMenu(state) => settings::menu::draw(ctx, state),
            App::NetworkSettings(state) => settings::network::draw(ctx, state),
        }
        Ok(())
    }

    pub fn main_menu() -> Self {
        Self::MainMenu(Default::default())
    }

    pub fn time_tools_menu() -> Self {
        Self::TimeToolsMenu(Default::default())
    }

    pub fn settings_menu() -> Self {
        Self::SettingsMenu(Default::default())
    }

    pub fn time_settings() -> Self {
        Self::TimeSettings(Default::default())
    }

    pub fn network_settings() -> Self {
        Self::NetworkSettings(Default::default())
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

    fn target_game() -> App {
        Self::Target(Default::default())
    }

    fn jump_jump_game() -> App {
        Self::JumpJump(Default::default())
    }

    fn whac_mole_game() -> App {
        Self::WhacMole(Default::default())
    }

    fn naval_battle_game() -> App {
        Self::NavalBattle(Default::default())
    }

    fn tank_trouble_game() -> App {
        Self::TankTrouble(Default::default())
    }

    fn game_2048() -> App {
        Self::Game2048(Default::default())
    }

    fn flappy_bird_game() -> App {
        Self::FlappyBird(Default::default())
    }

    fn gold_miner_game() -> App {
        Self::GoldMiner(Default::default())
    }

    fn gomoku_game() -> App {
        Self::Gomuku(Default::default())
    }

    fn brick_game() -> App {
        Self::Brick(Default::default())
    }

    fn undertale_game() -> App {
        Self::Undertale(Default::default())
    }

    fn pacman_game() -> App {
        Self::Pacman(Default::default())
    }
}
