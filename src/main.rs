use bevy::prelude::*;

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVER_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.20, 0.20, 0.20);

#[derive(Debug, Component)]
struct QuestionText;
#[derive(Debug, Component)]
struct AnswerBox;

#[derive(Debug, Resource)]
struct CanAnswer(bool);
#[derive(Debug, Resource)]
struct AnswerText(String);

fn main() {
    App::new()
        .add_event::<AnsweredEvent>()
        .insert_resource(CanAnswer(true))
        .insert_resource(AnswerText("Button 1!".into()))
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    meta_check: bevy::asset::AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Sinhala training".into(),
                        canvas: Some("#bevy".into()),
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: true,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_systems(Startup, spawn_text)
        .add_systems(Update, (button_system, handle_answer).chain())
        .run();
}

fn spawn_text(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    let toplevel = commands
        .spawn(NodeBundle {
            style: Style {
                display: Display::Flex,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        })
        .id();

    let top = commands
        .spawn(NodeBundle {
            style: Style {
                flex_grow: 1.0,
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::srgb(0.20, 0.20, 0.20).into(),
            ..default()
        })
        .with_children(|commands| {
            commands.spawn((
                QuestionText,
                TextBundle::from_section("Sinhala character!", default())
                    .with_text_justify(JustifyText::Center),
            ));
        })
        .id();

    let bottom = commands
        .spawn((
            AnswerBox,
            NodeBundle {
                style: Style {
                    flex_grow: 3.0,
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|commands| {
            for i in 0..5 {
                commands
                    .spawn(ButtonBundle {
                        style: Style {
                            width: Val::Px(150.0),
                            height: Val::Px(65.0),
                            border: UiRect::all(Val::Px(5.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        border_color: BorderColor(Color::BLACK),
                        background_color: NORMAL_BUTTON.into(),
                        ..default()
                    })
                    .with_children(|commands| {
                        commands.spawn(TextBundle::from_section(format!("Button {i}!"), default()));
                    });
            }
        })
        .id();

    commands.entity(toplevel).push_children(&[top, bottom]);
}

#[derive(Event)]
struct AnsweredEvent(pub Entity);

fn button_system(
    mut interaction_query: Query<
        (Entity, &Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut answered: EventWriter<AnsweredEvent>,
    can_answer: Res<CanAnswer>,
) {
    for (entity, interaction, mut color, mut border_color) in &mut interaction_query {
        if can_answer.0 {
            match *interaction {
                Interaction::Pressed => {
                    //text.sections[0].value = "Press".to_string();
                    *color = PRESSED_BUTTON.into();
                    //border_color.0 = Color::srgb(1.0, 0.0, 0.0);
                    answered.send(AnsweredEvent(entity));
                }
                Interaction::Hovered => {
                    //text.sections[0].value = "Hover".to_string();
                    *color = HOVER_BUTTON.into();
                    border_color.0 = Color::WHITE;
                }
                Interaction::None => {
                    //text.sections[0].value = "Button".to_string();
                    *color = NORMAL_BUTTON.into();
                    border_color.0 = Color::BLACK;
                }
            }
        }
    }
}

fn handle_answer(
    children: Query<&Children>,
    mut buttons: Query<(Entity, &mut BackgroundColor, &mut BorderColor), With<Button>>,
    text: Query<&Text>,
    mut answered: ResMut<Events<AnsweredEvent>>,
    mut can_answer: ResMut<CanAnswer>,
    answer_text: Res<AnswerText>,
) {
    for AnsweredEvent(answered_entity) in answered.drain().take(1) {
        *can_answer = CanAnswer(false);

        let answer = &text
            .get(children.get(answered_entity).unwrap()[0])
            .unwrap()
            .sections[0]
            .value;
        println!("Answered: {answer}, answer: {}", answer_text.0);

        for (entity, mut color, mut border_color) in &mut buttons {
            if entity == answered_entity {
                *color = PRESSED_BUTTON.into();
                if answer == &answer_text.0 {
                    border_color.0 = Color::srgb(0.0, 1.0, 0.0);
                } else {
                    border_color.0 = Color::srgb(1.0, 0.0, 0.0);
                }
            } else {
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}
