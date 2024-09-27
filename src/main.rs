use bevy::prelude::*;
use rand::seq::IteratorRandom;
use std::collections::BTreeMap;

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
struct Question(String);
#[derive(Debug, Resource)]
struct Questions(BTreeMap<String, String>);

#[derive(Event)]
struct AnsweredEvent(pub Entity);
#[derive(Event)]
struct RestartEvent;

fn main() {
    let questions = BTreeMap::<String, String>::from([
        ("ක".into(), "ka".into()),
        ("කි".into(), "ki".into()),
        ("කැ".into(), "kæ".into()),
        ("කු".into(), "ku".into()),
        ("කෙ".into(), "ke".into()),
        ("කො".into(), "ko".into()),
        ("ක්".into(), "k".into()),
        ("බ".into(), "ba".into()),
        ("ච".into(), "ca".into()),
        ("ජ".into(), "ja".into()),
        ("ට".into(), "ṭa".into()),
        ("බැ".into(), "bæ".into()),
        ("බි".into(), "bi".into()),
        ("බු".into(), "bu".into()),
        ("බෙ".into(), "be".into()),
        ("බො".into(), "bo".into()),
        ("බ්".into(), "b".into()),
        ("කා".into(), "kā".into()),
        ("කෑ".into(), "kǣ".into()),
        ("කී".into(), "kī".into()),
        ("කූ".into(), "kū".into()),
        ("කේ".into(), "kē".into()),
        ("කෝ".into(), "kō".into()),
        ("ත".into(), "ta".into()),
        ("ඩ".into(), "ḍa".into()),
    ]);

    let mut thread_rng = rand::thread_rng();
    let question = questions.keys().choose(&mut thread_rng).unwrap().into();
    //let question: String  = "කො".into();

    App::new()
        .add_event::<AnsweredEvent>()
        .add_event::<RestartEvent>()
        .insert_resource(CanAnswer(true))
        .insert_resource(Question(question))
        .insert_resource(Questions(questions))
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
        .add_systems(
            Update,
            (
                reset_one_second_after_answer,
                setup_question,
                button_system,
                handle_answer,
            )
                .chain(),
        )
        .run();
}

fn reset_one_second_after_answer(
    mut could_answer: Local<f32>,
    time: Res<Time>,
    can_answer: Res<CanAnswer>,
    mut event_writer: EventWriter<RestartEvent>,
) {
    if can_answer.0 {
        *could_answer = time.elapsed_seconds();
    } else if *could_answer + 1.0 < time.elapsed_seconds() {
        event_writer.send(RestartEvent);
        *could_answer = time.elapsed_seconds();
    }
}

fn setup_question(
    mut event_reader: EventReader<RestartEvent>,
    mut question_text: Query<&mut Text, With<QuestionText>>,
    mut question: ResMut<Question>,
    mut can_answer: ResMut<CanAnswer>,
    mut buttons: Query<(&mut BackgroundColor, &mut BorderColor), With<Button>>,
    questions: Res<Questions>,
) {
    for _ in event_reader.read() {
        let mut question_text = question_text.single_mut();
        let mut thread_rng = rand::thread_rng();
        let new_question = questions.0.keys().choose(&mut thread_rng).unwrap();
        question_text.sections[0].value = new_question.into();
        question.0 = new_question.into();

        can_answer.0 = true;

        for (mut color, mut border_color) in &mut buttons {
            color.0 = NORMAL_BUTTON;
            border_color.0 = Color::BLACK;
        }
    }
}

fn spawn_text(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    question: Res<Question>,
    questions: Res<Questions>,
) {
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
                TextBundle::from_section(
                    &question.0,
                    TextStyle {
                        font: asset_server.load(
                            "fonts/Noto_Sans_Sinhala/NotoSansSinhala-VariableFont_wdth,wght.ttf",
                        ),
                        font_size: 75.0,
                        ..default()
                    },
                )
                .with_text_justify(JustifyText::Center),
            ));
        })
        .id();

    let bottom = commands
        .spawn((
            AnswerBox,
            NodeBundle {
                style: Style {
                    display: Display::Grid,
                    flex_grow: 3.0,
                    width: Val::Percent(100.0),
                    grid_template_rows: vec![RepeatedGridTrack::percent(5, (100.0 - 2.0) / 5.0)],
                    grid_template_columns: vec![RepeatedGridTrack::percent(5, (100.0 - 2.0) / 5.0)],
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|commands| {
            for answer in questions.0.values() {
                commands
                    .spawn(ButtonBundle {
                        style: Style {
                            //width: Val::Px(150.0),
                            //height: Val::Px(65.0),
                            border: UiRect::all(Val::Px(5.0)),
                            margin: UiRect::all(Val::Px(10.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        border_color: BorderColor(Color::BLACK),
                        background_color: NORMAL_BUTTON.into(),
                        ..default()
                    })
                    .with_children(|commands| {
                        commands.spawn(TextBundle::from_section(
                            answer,
                            TextStyle {
                                font: asset_server
                                    .load("fonts/Noto_Serif/NotoSerif-VariableFont_wdth,wght.ttf"),
                                font_size: 75.0,
                                ..default()
                            },
                        ));
                    });
            }
        })
        .id();

    commands.entity(toplevel).add_children(&[top, bottom]);
}

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
    mut buttons: Query<(Entity, &mut BackgroundColor, &mut BorderColor, &Children), With<Button>>,
    text: Query<&Text>,
    mut answered: ResMut<Events<AnsweredEvent>>,
    mut can_answer: ResMut<CanAnswer>,
    question: Res<Question>,
    questions: Res<Questions>,
) {
    for AnsweredEvent(answered_entity) in answered.drain().take(1) {
        *can_answer = CanAnswer(false);

        let answer = &text
            .get(children.get(answered_entity).unwrap()[0])
            .unwrap()
            .sections[0]
            .value;
        let correct_answer = questions.0.get(&question.0).unwrap();
        let correct_entity = buttons
            .iter()
            .find(|(_, _, _, children)| {
                &text.get(children[0]).unwrap().sections[0].value == correct_answer
            })
            .unwrap()
            .0;
        println!("Answered: {answer}, correct answer: {correct_answer}");

        for (entity, mut color, mut border_color, _) in &mut buttons {
            if entity == answered_entity {
                *color = PRESSED_BUTTON.into();
                if answer == correct_answer {
                    border_color.0 = Color::srgb(0.0, 1.0, 0.0);
                } else {
                    border_color.0 = Color::srgb(1.0, 0.0, 0.0);
                }
            } else if entity == correct_entity {
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::srgb(0.0, 0.0, 1.0);
            } else {
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}
