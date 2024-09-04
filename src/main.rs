use std::time::Duration;

use bevy::prelude::*;
use bevy_asset::{AssetServer, Handle};
use bevy_rapier2d::prelude::*;
use bevy_tweening::{
    lens::UiBackgroundColorLens,
    Animator,
    AnimatorState,
    EaseFunction,
    RepeatCount,
    RepeatStrategy,
    Tween,
    TweeningPlugin,
};
use leafwing_input_manager::prelude::*;
mod game;
use bevy_screen_diagnostics::{ScreenDiagnosticsPlugin, ScreenFrameDiagnosticsPlugin};
#[cfg(feature = "discord")]
use discord_game_sdk::Discord;
use game::{ball_collision_detection, movement, setup_game};
use leafwing_input_manager::Actionlike;
#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
enum Action {
    Left,
    Right,
    Up,
    Down,
    RotateClockwise,
    RotateAntiClockwise,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Default, States)]
enum AppState {
    #[default]
    Setup,
    MainMenu,
    InGame,
    Paused,
}
#[cfg(feature = "discord")]
struct DiscordState<'a>(Discord<'a, ()>);

#[derive(PartialEq, Reflect, Default, Clone, Debug)]
enum RotatingM {
    Clockwise,
    AntiClockwise,
    #[default]
    Neither,
}

#[derive(Component)]
struct Paddle;
#[derive(Component)]
struct Ball;

#[derive(Component)]
struct TopLevelNode;

#[derive(Component)]
struct RotationVelocity(f32);

#[derive(Component)]
struct Acceleration(f32);
#[derive(Component)]
struct RotAcceleration(f32);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct NextStop(f32);

#[derive(Reflect, Default)]
#[reflect(Component)]
#[derive(Component)]
struct Rotating(RotatingM);

#[derive(Bundle)]
struct PaddleBundle {
    flags: ActiveEvents,
    active_collision_types: ActiveCollisionTypes,
    rotation_velocity: RotationVelocity,
    acceleration: Acceleration,
    rot_acceleration: RotAcceleration,
    next_stop: NextStop,
    rotating: Rotating,
    sprite: SpriteBundle,
}
#[derive(Component)]
struct MenuButtonId(Option<String>);

#[derive(Event)]
struct MenuButtonPressed(String);

#[derive(Bundle)]
struct MenuButtonBundle {
    node: NodeBundle,
    mbid: MenuButtonId,
}

fn main() {
    let mut app = App::new();
    app.add_plugins(TweeningPlugin)
        .add_plugins(
            DefaultPlugins.set(AssetPlugin {
                watch_for_changes_override: None,
                file_path: if cfg!(target_os = "windows")
                    || cfg!(target_os = "linux")
                    || cfg!(debug_assertions)
                {
                    "assets"
                } else if cfg!(target_os = "macos") {
                    "../Resources/assets"
                } else {
                    panic!("unsupported os")
                }
                .to_string(),
                processed_file_path: "proc_assets".to_string(),
                mode: AssetMode::Unprocessed,
            }),
        )
        .add_plugins(ScreenDiagnosticsPlugin::default())
        .add_plugins(ScreenFrameDiagnosticsPlugin)
        // .add_plugins(EguiPlugin)
        .add_plugins(InputManagerPlugin::<Action>::default())
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(50.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_state::<AppState>();

    #[cfg(debug_assertions)]
    {
        use bevy_inspector_egui::quick::WorldInspectorPlugin;
        app.add_plugins(WorldInspectorPlugin::new());
    }

    // events

    app.add_event::<MenuButtonPressed>();

    // misc systems
    app.add_systems(Startup, setup)
        .add_systems(Update, input_system);

    // game systems
    app.add_systems(OnEnter(AppState::InGame), setup_game)
        .add_systems(Update, movement.run_if(in_state(AppState::InGame)))
        .add_systems(
            Update,
            ball_collision_detection.run_if(in_state(AppState::InGame)),
        );

    // menu systems
    app.add_systems(OnEnter(AppState::MainMenu), menu_menu)
        .add_systems(Update, menu_update.run_if(in_state(AppState::MainMenu)))
        // exit menu
        .add_systems(
            OnExit(AppState::MainMenu),
            |mut commands: Commands, query: Query<(Entity, With<TopLevelNode>)>| {
                for entity in query.iter() {
                    // Remove the entity if it has MenuButtonId and Button components
                    commands.entity(entity.0).despawn_recursive();
                }
            },
        );
    // // setup systems
    // app.add_systems(OnEnter(AppState::Setup), setup_menu)
    //     .add_systems(Update, setup_update.run_if(in_state(AppState::Setup)))
    //     // exit menu
    //     .add_systems(
    //         OnExit(AppState::Setup),
    //         |mut commands: Commands, query: Query<(Entity, With<TopLevelNode>)>| {
    //             for entity in query.iter() {
    //                 // Remove the entity if it has MenuButtonId and Button components
    //                 commands.entity(entity.0).despawn_recursive();
    //             }
    //         },
    //     );
    // discord
    #[cfg(feature = "discord")]
    app.add_systems(Startup, setup_discord)
        .add_systems(Update, discord_update);

    app.run();
}

const NORMAL_BUTTON_COLOR: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON_COLOR: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

fn setup(
    mut commands: Commands,
    mut rapier_config: ResMut<RapierConfiguration>,
    server: Res<AssetServer>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    rapier_config.gravity = Vec2::new(0.0, 0.0);
    commands.spawn(Camera2dBundle::default());
    // server.().unwrap();

    // server_settings.asset_folder =
    // // changes for each os
    // if cfg!(target_os = "windows") || cfg!(target_os = "linux") || cfg!(debug_assertions){
    //     "assets"
    // } else if cfg!(target_os = "macos") {
    //     "../Resources/assets"
    // } else {
    //     panic!("unsupported os")
    // }.to_string();
    let _blazma: Handle<Font> = server.load("Blazma/Blazma-Regular.ttf");
    let _noto_sans: Handle<Font> =
        server.load("Noto_Sans_Mono/NotoSansMono-VariableFont_wdth,wght.ttf");

    next_state.set(AppState::MainMenu);
}

fn menu_menu(mut commands: Commands, server: Res<AssetServer>) {
    let blazma: Handle<Font> = server.load("Blazma/Blazma-Regular.ttf");

    // spawn node bundle for buttons
    let node = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
            TopLevelNode,
        ))
        .id();

    spawn_menu_button(
        &mut commands,
        node,
        "STEM",
        blazma.clone(),
        Some("test_start_game".to_string()),
    );
    spawn_menu_button(
        &mut commands,
        node,
        "custom",
        blazma.clone(),
        Some("custom".to_string()),
    );
}

fn spawn_menu_button(
    commands: &mut Commands,
    node: Entity,
    text: &str,
    font: Handle<Font>,
    mbid: Option<String>,
) -> Entity {
    // Create a single animation (tween) to move an entity.
    let hover_tween = Tween::new(
        // Use a quadratic easing on both endpoints.
        EaseFunction::QuadraticInOut,
        // Animation time (one way only; for ping-pong it takes 2 seconds
        // to come back to start).
        Duration::from_secs(1),
        // The lens gives the Animator access to the Transform component,
        // to animate it. It also contains the start and end values associated
        // with the animation ratios 0. and 1.
        UiBackgroundColorLens {
            end: NORMAL_BUTTON_COLOR,
            start: HOVERED_BUTTON_COLOR,
        },
    )
    // Repeat twice (one per way)
    .with_repeat_count(RepeatCount::Infinite)
    // After each iteration, reverse direction (ping-pong)
    .with_repeat_strategy(RepeatStrategy::MirroredRepeat);
    commands
        .entity(node)
        .with_children(|parent| {
            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(150.0),
                            height: Val::Px(65.0),
                            // horizontally center child text
                            justify_content: JustifyContent::Center,
                            // vertically center child text
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: NORMAL_BUTTON_COLOR.into(),
                        ..default()
                    },
                    MenuButtonId(mbid),
                    Animator::new(hover_tween),
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        text,
                        TextStyle {
                            font,
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    ));
                });
        })
        .id()
}

fn menu_update(
    _commands: Commands,
    mut menu_button_pressed: EventReader<MenuButtonPressed>,
    _app_state: ResMut<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for MenuButtonPressed(id) in &mut menu_button_pressed.read() {
        match id.as_str() {
            "test_start_game" => next_state.set(AppState::InGame),
            "custom" => next_state.set(AppState::Setup),
            _ => {}
        }
    }
}

fn input_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &Children,
            &MenuButtonId,
            &mut Animator<BackgroundColor>,
            &mut BackgroundColor,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
    _app_state: ResMut<NextState<AppState>>,
    mut menu_pressed: EventWriter<MenuButtonPressed>,
) {
    for (interaction, children, mbid, mut anim, mut color) in &mut interaction_query {
        let _text = text_query.get_mut(children[0]).unwrap();
        // let target;
        match *interaction {
            Interaction::Pressed => {
                // target = PRESSED_BUTTON.into();
                if let Some(id) = &mbid.0 {
                    menu_pressed.send(MenuButtonPressed(id.clone()));
                }
            }
            Interaction::Hovered => {
                // target = HOVERED_BUTTON.into();
                anim.state = AnimatorState::Playing
            }
            Interaction::None => {
                *color = NORMAL_BUTTON_COLOR.into();
                anim.stop();
            }
        };
    }
}

#[cfg(feature = "discord")]
fn setup_discord(world: &mut World) {
    let discord = Discord::with_create_flags(
        1_023_380_299_821_875_210,
        discord_game_sdk::CreateFlags::NoRequireDiscord,
    );
    match discord {
        Ok(discord) => {
            world.insert_non_send_resource(DiscordState(discord));
        }
        Err(_e) => {
            println!("warning: discord setup failed...");
        }
    }
}

#[cfg(feature = "discord")]
fn discord_update(discord: Option<NonSendMut<DiscordState>>) {
    // TODO: add more states
    if discord.is_none() {
        return;
    }
    let mut discord = discord.unwrap();
    discord.0.run_callbacks().unwrap();
    let mut activity = discord_game_sdk::Activity::empty();
    let activity = activity
        // party status
        .with_state("idle")
        .with_large_image_key("logo")
        .with_large_image_key("logo")
        // player status
        .with_details("in the menus");
    discord.0.update_activity(activity, |_, result| {
        if let Err(e) = result {
            println!("Error updating activity: {}", e);
        }
    });
}
