use crate::level::LevelState;
use crate::AppState;
use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;

pub struct MenuPlugin;

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (manage_main_menu, manage_pause_menu))
            .add_systems(Update, button_background)
            .add_systems(Update, start_listener)
            .add_systems(Update, (resume_listener, main_menu_listener));
    }
}

#[derive(Default, Debug, Copy, Clone, Component)]
pub struct MainMenu;

#[derive(Default, Debug, Copy, Clone, Component)]
pub struct StartGameButton;

#[derive(Default, Debug, Copy, Clone, Component)]
pub struct PauseMenu;

#[derive(Default, Debug, Copy, Clone, Component)]
pub struct ResumeGameButton;

#[derive(Default, Debug, Copy, Clone, Component)]
pub struct MainMenuButton;

fn manage_main_menu(
    app_state: Res<State<AppState>>,
    main_menu_query: Query<Entity, With<MainMenu>>,
    mut commands: Commands,
    assets: Res<AssetServer>,
) {
    if app_state.is_changed() {
        if *app_state.get() == AppState::MainMenu {
            if main_menu_query.is_empty() {
                commands
                    .spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        ..default()
                    })
                    .insert(MainMenu)
                    .with_children(|parent| {
                        spawn_button(
                            parent,
                            Val::Px(150.0), Val::Px(65.0),
                            "Load",
                            &assets,
                        )
                        .insert(StartGameButton);
                    });
            }
        } else {
            if let Some(menu) = main_menu_query.iter().next() {
                commands.entity(menu).despawn_recursive();
            }
        }
    }
}

fn manage_pause_menu(
    app_state: Res<State<AppState>>,
    pause_menu_query: Query<Entity, With<PauseMenu>>,
    mut commands: Commands,
    assets: Res<AssetServer>,
) {
    if app_state.is_changed() {
        if *app_state.get() == AppState::PauseMenu {
            if pause_menu_query.is_empty() {
                commands
                    .spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        ..default()
                    })
                    .insert(PauseMenu)
                    .with_children(|parent| {
                        spawn_button(
                            parent,
                            Val::Px(200.0), Val::Px(65.0),
                            "Resume",
                            &assets,
                        )
                        .insert(ResumeGameButton);
                        spawn_button(
                            parent,
                            Val::Px(200.0), Val::Px(65.0),
                            "Main Menu",
                            &assets,
                        )
                        .insert(MainMenuButton);
                    });
            }
        } else {
            if let Some(menu) = pause_menu_query.iter().next() {
                commands.entity(menu).despawn_recursive();
            }
        }
    }
}

fn spawn_button<'a, 'w, 's>(
    parent: &'a mut ChildBuilder<'w, 's, '_>,
    width: Val,
    height: Val,
    text: impl Into<String>,
    assets: &Res<AssetServer>,
) -> EntityCommands<'w, 's, 'a> {
    let mut commands = parent.spawn(ButtonBundle {
        background_color: NORMAL_BUTTON.into(),
        style: Style {
            width,
            height,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        ..default()
    });
    commands.with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            text,
            TextStyle {
                font: assets.load("fonts/FiraSans-Bold.ttf"),
                font_size: 40.0,
                color: Color::rgb(0.9, 0.9, 0.9),
                ..default()
            },
        ));
    });
    commands
}

fn button_background(
    mut buttons: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, mut color) in buttons.iter_mut() {
        match interaction {
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
            Interaction::Pressed => {}
        }
    }
}

fn start_listener(
    start_game: Query<&Interaction, (Changed<Interaction>, With<StartGameButton>)>,
    mut level_state: ResMut<LevelState>,
    mut app_state: ResMut<NextState<AppState>>,
    assets: Res<AssetServer>,
) {
    for &interaction in start_game.iter() {
        if interaction == Interaction::Pressed {
            app_state.set(AppState::Loading);
            level_state.handle = Some(assets.load("levels/level0.level.kdl"));
            return;
        }
    }
}

fn resume_listener(
    resume_game: Query<&Interaction, (Changed<Interaction>, With<ResumeGameButton>)>,
    mut app_state: ResMut<NextState<AppState>>,
) {
    for &interaction in resume_game.iter() {
        if interaction == Interaction::Pressed {
            app_state.set(AppState::InGame);
            return;
        }
    }
}

fn main_menu_listener(
    main_menu: Query<&Interaction, (Changed<Interaction>, With<MainMenuButton>)>,
    mut level_state: ResMut<LevelState>,
    mut app_state: ResMut<NextState<AppState>>,
) {
    for &interaction in main_menu.iter() {
        if interaction == Interaction::Pressed {
            app_state.set(AppState::MainMenu);
            level_state.handle = None;
            return;
        }
    }
}
