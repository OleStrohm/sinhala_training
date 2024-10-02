use bevy::prelude::*;
use rand::{prelude::SliceRandom, seq::IteratorRandom};

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVER_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.20, 0.20, 0.20);

#[derive(Debug, Component)]
struct QuestionText;
#[derive(Debug, Component)]
struct AnswerText;
#[derive(Debug, Component)]
struct AnswerBox;
#[derive(Debug, Component)]
struct AnswerButton;
#[derive(Debug, Component)]
enum SettingsButton {
    SwitchDirection,
    RerollQuestions,
}

#[derive(Debug, Resource, Clone, Copy)]
enum TranslateDirection {
    SinhalaToEnglish,
    EnglishToSinhala,
}
impl TranslateDirection {
    pub fn question_font(&self, asset_server: &AssetServer) -> Handle<Font> {
        match *self {
            TranslateDirection::SinhalaToEnglish => asset_server
                .load("fonts/Noto_Sans_Sinhala/NotoSansSinhala-VariableFont_wdth,wght.ttf"),
            TranslateDirection::EnglishToSinhala => {
                asset_server.load("fonts/Noto_Serif/NotoSerif-VariableFont_wdth,wght.ttf")
            }
        }
    }

    pub fn answer_font(&self, asset_server: &AssetServer) -> Handle<Font> {
        match *self {
            TranslateDirection::SinhalaToEnglish => {
                asset_server.load("fonts/Noto_Serif/NotoSerif-VariableFont_wdth,wght.ttf")
            }
            TranslateDirection::EnglishToSinhala => asset_server
                .load("fonts/Noto_Sans_Sinhala/NotoSansSinhala-VariableFont_wdth,wght.ttf"),
        }
    }
}
#[derive(Debug, Resource, Deref, DerefMut)]
struct CanAnswer(bool);
#[derive(Debug, Resource, Deref, DerefMut, PartialEq, Eq)]
struct Question(Pair);
#[derive(Debug, Resource, Deref, DerefMut)]
struct Questions(Vec<Pair>);
#[derive(Debug, Resource, Deref, DerefMut)]
struct AllQuestions(Vec<Pair>);

#[derive(Event)]
struct AnsweredEvent(pub Entity);
#[derive(Event)]
struct RestartEvent;
#[derive(Event)]
struct RerollQuestionsEvent;

use pair::Pair;
mod pair {
    use super::TranslateDirection;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Pair {
        sinhala: String,
        english: String,
    }

    impl Pair {
        pub fn question(&self, translation_direction: TranslateDirection) -> String {
            match translation_direction {
                TranslateDirection::SinhalaToEnglish => self.sinhala.clone(),
                TranslateDirection::EnglishToSinhala => self.english.clone(),
            }
        }

        pub fn answer(&self, translation_direction: TranslateDirection) -> String {
            match translation_direction {
                TranslateDirection::SinhalaToEnglish => self.english.clone(),
                TranslateDirection::EnglishToSinhala => self.sinhala.clone(),
            }
        }
    }

    impl From<(&str, &str)> for Pair {
        fn from((sinhala, english): (&str, &str)) -> Self {
            Self {
                sinhala: sinhala.into(),
                english: english.into(),
            }
        }
    }
}

fn main() {
    let all_questions = Vec::<Pair>::from([
        ("ක", "ka").into(),
        ("ඛ", "kha").into(),
        ("ග", "ga").into(),
        ("ඝ", "gha").into(),
        ("ඞ", "ṅa").into(),
        ("ච", "ca").into(),
        ("ඡ", "cha").into(),
        ("ජ", "ja").into(),
        ("ඣ", "jha").into(),
        ("ඤ", "ñ").into(),
        ("ට", "ṭa").into(),
        ("ඨ", "ṭha").into(),
        ("ඩ", "ḍa").into(),
        ("ඪ", "ḍha").into(),
        ("ණ", "ṇa").into(),
        ("ත", "ta").into(),
        ("ථ", "tha").into(),
        ("ද", "da").into(),
        ("ධ", "dha").into(),
        ("න", "na").into(),
        ("ප", "pa").into(),
        ("ඵ", "pha").into(),
        ("බ", "ba").into(),
        ("භ", "bha").into(),
        ("ම", "ma").into(),
        ("ය", "ya").into(),
        ("ර", "ra").into(),
        ("ල", "la").into(),
        ("ව", "va").into(),
        ("ශ", "śa").into(),
        ("ෂ", "ṣa").into(),
        ("ස", "sa").into(),
        ("හ", "ha").into(),
        ("ඥ", "jña").into(),
        ("ළ", "ḷa").into(),
        ("ෆ", "fa").into(),
        ("ඟ", "n̆ga").into(),
        ("ඦ", "n̆ja").into(),
        ("ඬ", "n̆ḍa").into(),
        ("ඳ", "n̆da").into(),
        ("ඹ", "m̆ba").into(),
        ("අ", "a").into(),
        ("ඇ", "æ").into(),
        ("ඉ", "i").into(),
        ("උ", "u").into(),
        ("එ", "e").into(),
        ("ඔ", "o").into(),
    ]);

    let mut thread_rng = rand::thread_rng();
    let questions = all_questions.iter().take(25).cloned().collect::<Vec<_>>();
    let question = questions.iter().choose(&mut thread_rng).unwrap().clone();

    App::new()
        .add_event::<AnsweredEvent>()
        .add_event::<RestartEvent>()
        .add_event::<RerollQuestionsEvent>()
        .insert_resource(CanAnswer(true))
        .insert_resource(Question(question))
        .insert_resource(Questions(questions))
        .insert_resource(AllQuestions(all_questions))
        .insert_resource(TranslateDirection::SinhalaToEnglish)
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
                settings_button_system,
                reset_one_second_after_answer,
                reroll_questions,
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

fn reroll_questions(
    mut event_reader: EventReader<RerollQuestionsEvent>,
    all_questions: Res<AllQuestions>,
    mut questions: ResMut<Questions>,
    mut event_writer: EventWriter<RestartEvent>,
) {
    for _ in event_reader.read() {
        let mut thread_rng = rand::thread_rng();
        questions.0 = all_questions
            .0
            .iter()
            .cloned()
            .choose_multiple(&mut thread_rng, 25);
        questions.0.shuffle(&mut thread_rng);
        event_writer.send(RestartEvent);
    }
}

fn setup_question(
    mut event_reader: EventReader<RestartEvent>,
    mut question_text: Query<&mut Text, (With<QuestionText>, Without<AnswerText>)>,
    mut question: ResMut<Question>,
    mut can_answer: ResMut<CanAnswer>,
    mut buttons: Query<(&mut BackgroundColor, &mut BorderColor), With<AnswerButton>>,
    mut answer_texts: Query<(Entity, &mut Text), (With<AnswerText>, Without<QuestionText>)>,
    questions: Res<Questions>,
    translation_direction: Res<TranslateDirection>,
    asset_server: Res<AssetServer>,
) {
    for _ in event_reader.read() {
        let mut thread_rng = rand::thread_rng();

        let mut question_text = question_text.single_mut();
        let new_question = questions
            .iter()
            .filter(|&q| q != &question.0)
            .choose(&mut thread_rng)
            .unwrap()
            .clone();
        question_text.sections[0].value = new_question.question(*translation_direction);
        question_text.sections[0].style.font = translation_direction.question_font(&asset_server);
        question.0 = new_question;

        can_answer.0 = true;

        for (mut color, mut border_color) in &mut buttons {
            color.0 = NORMAL_BUTTON;
            border_color.0 = Color::BLACK;
        }

        let mut answer_text_entities = answer_texts.iter().map(|(e, _)| e).collect::<Vec<_>>();
        answer_text_entities.sort();
        for (q, e) in questions.iter().zip(answer_text_entities) {
            let section = &mut answer_texts.get_mut(e).unwrap().1.sections[0];
            section.value = q.answer(*translation_direction);
            section.style.font = translation_direction.answer_font(&asset_server);
        }
    }
}

fn spawn_text(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut restart: EventWriter<RestartEvent>,
    questions: Res<Questions>,
) {
    restart.send(RestartEvent);

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
                display: Display::Grid,
                width: Val::Percent(100.0),
                grid_template_rows: vec![RepeatedGridTrack::percent(1, 100.0)],
                grid_template_columns: vec![RepeatedGridTrack::percent(3, 100.0 / 3.0)],
                ..default()
            },
            background_color: Color::srgb(0.20, 0.20, 0.20).into(),
            ..default()
        })
        .with_children(|commands| {
            commands
                .spawn(NodeBundle {
                    style: Style {
                        display: Display::Flex,
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Start,
                        align_items: AlignItems::Start,
                        padding: UiRect::all(Val::Px(5.0)),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|commands| {
                    commands
                        .spawn((
                            SettingsButton::SwitchDirection,
                            ButtonBundle {
                                style: Style {
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                ..default()
                            },
                        ))
                        .with_children(|commands| {
                            commands.spawn((TextBundle::from_section(
                                "ක -> ka",
                                TextStyle {
                                    font: asset_server.load(
                                        "fonts/Noto_Sans_Sinhala/NotoSansSinhala-VariableFont_wdth,wght.ttf",
                                    ),
                                    font_size: 50.0,
                                    ..default()
                                },
                            ),));
                        });
                });

            commands.spawn(NodeBundle {
                style: Style {
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            })
            .with_children(|commands| {
                commands.spawn((
                    QuestionText,
                    TextBundle::from_section(
                        "",
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
            });

            commands.spawn(NodeBundle {
                style: Style {
                    display: Display::Flex,
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::FlexEnd,
                    padding: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
                ..default()
            }) .with_children(|commands| {
                commands
                    .spawn((
                        SettingsButton::RerollQuestions,
                        ButtonBundle {
                            style: Style {
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            ..default()
                        },
                    ))
                    .with_children(|commands| {
                        commands.spawn((TextBundle::from_section(
                            " ",
                            TextStyle {
                                font: asset_server.load(
                                    "fonts/0xProto/0xProtoNerdFont-Regular.ttf",
                                ),
                                font_size: 50.0,
                                ..default()
                            },
                        ),));
                    });
            });
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
                    grid_template_rows: vec![RepeatedGridTrack::percent(5, 20.0)],
                    grid_template_columns: vec![RepeatedGridTrack::percent(5, 20.0)],
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|commands| {
            for _ in &questions.0 {
                commands
                    .spawn((
                        AnswerButton,
                        ButtonBundle {
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
                        },
                    ))
                    .with_children(|commands| {
                        commands.spawn((
                            AnswerText,
                            TextBundle::from_section(
                                "",
                                TextStyle {
                                    font: asset_server.load(
                                        "fonts/Noto_Serif/NotoSerif-VariableFont_wdth,wght.ttf",
                                    ),
                                    font_size: 75.0,
                                    ..default()
                                },
                            ),
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
        (Changed<Interaction>, With<AnswerButton>),
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

fn settings_button_system(
    mut interaction_query: Query<(&Interaction, &Children, &SettingsButton), Changed<Interaction>>,
    mut text: Query<&mut Text>,
    mut reroll_questions: EventWriter<RerollQuestionsEvent>,
    mut restart: EventWriter<RestartEvent>,
    mut translation_direction: ResMut<TranslateDirection>,
) {
    for (interaction, children, setting) in &mut interaction_query {
        let text = &mut text.get_mut(children[0]).unwrap().sections[0];
        text.style.color = match *interaction {
            Interaction::Pressed => Color::srgb(1.0, 1.0, 1.0),
            Interaction::Hovered => Color::srgb(0.9, 0.9, 0.9),
            Interaction::None => Color::srgb(0.8, 0.8, 0.8),
        };
        if *interaction == Interaction::Pressed {
            match setting {
                SettingsButton::SwitchDirection => {
                    *translation_direction = match *translation_direction {
                        TranslateDirection::SinhalaToEnglish => {
                            text.value = "ක <- ka".into();
                            TranslateDirection::EnglishToSinhala
                        }
                        TranslateDirection::EnglishToSinhala => {
                            text.value = "ක -> ka".into();
                            TranslateDirection::SinhalaToEnglish
                        }
                    };
                    restart.send(RestartEvent);
                }
                SettingsButton::RerollQuestions => {
                    reroll_questions.send(RerollQuestionsEvent);
                }
            }
        }
    }
}

fn handle_answer(
    children: Query<&Children>,
    mut buttons: Query<
        (Entity, &mut BackgroundColor, &mut BorderColor, &Children),
        With<AnswerButton>,
    >,
    text: Query<&Text>,
    mut answered: ResMut<Events<AnsweredEvent>>,
    mut can_answer: ResMut<CanAnswer>,
    question: Res<Question>,
    translation_direction: Res<TranslateDirection>,
) {
    for AnsweredEvent(answered_entity) in answered.drain().take(1) {
        *can_answer = CanAnswer(false);

        let answer = &text
            .get(children.get(answered_entity).unwrap()[0])
            .unwrap()
            .sections[0]
            .value;
        let correct_answer = question.answer(*translation_direction);
        let correct_entity = buttons
            .iter()
            .find(|(_, _, _, children)| {
                text.get(children[0]).unwrap().sections[0].value == correct_answer
            })
            .unwrap()
            .0;
        println!(
            "Question: {}, Answered: {answer}, correct answer: {correct_answer}",
            question.question(*translation_direction)
        );

        for (entity, mut color, mut border_color, _) in &mut buttons {
            if entity == answered_entity {
                *color = PRESSED_BUTTON.into();
                if answer == &correct_answer {
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
