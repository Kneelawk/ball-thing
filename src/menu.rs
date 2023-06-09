use crate::level::LevelState;
use crate::AppState;
use bevy::prelude::*;

pub struct MenuPlugin;

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_main_menu)
            .add_system(button_background)
            .add_system(start.in_base_set(CoreSet::PostUpdate));
    }
}

#[derive(Default, Debug, Copy, Clone, Component)]
pub struct StartGame;

#[derive(Default, Debug, Copy, Clone, Component)]
pub struct MainMenu;

fn spawn_main_menu(
    app_state: Res<State<AppState>>,
    main_menu_query: Query<(), With<MainMenu>>,
    mut commands: Commands,
    assets: Res<AssetServer>,
) {
    if app_state.is_changed() && app_state.0 == AppState::MainMenu && main_menu_query.is_empty() {
        commands
            .spawn(NodeBundle {
                style: Style {
                    size: Size::width(Val::Percent(100.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            })
            .insert(MainMenu)
            .with_children(|parent| {
                parent
                    .spawn(ButtonBundle {
                        background_color: NORMAL_BUTTON.into(),
                        style: Style {
                            size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        ..default()
                    })
                    .insert(StartGame)
                    .with_children(|parent| {
                        parent.spawn(TextBundle::from_section(
                            "Load",
                            TextStyle {
                                font: assets.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 40.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                                ..default()
                            },
                        ));
                    });
            });
    }
}

fn button_background(
    mut buttons: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, mut color) in buttons.iter_mut() {
        match interaction {
            Interaction::Clicked => {}
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

fn start(
    start_game: Query<&Interaction, (Changed<Interaction>, With<StartGame>)>,
    main_menu: Query<Entity, With<MainMenu>>,
    mut commands: Commands,
    mut level_state: ResMut<LevelState>,
    mut app_state: ResMut<NextState<AppState>>,
    assets: Res<AssetServer>,
) {
    for &interaction in start_game.iter() {
        if interaction == Interaction::Clicked {
            commands.entity(main_menu.single()).despawn_recursive();
            app_state.set(AppState::Loading);
            level_state.handle = Some(assets.load("levels/level0.level.kdl"));
        }
    }
}
