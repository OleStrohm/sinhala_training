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
        *could_answer = time.elapsed_secs();
    } else if *could_answer + 1.0 < time.elapsed_secs() {
        event_writer.send(RestartEvent);
        *could_answer = time.elapsed_secs();
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
    mut question_text: Query<(&mut Text, &mut TextFont), (With<QuestionText>, Without<AnswerText>)>,
    mut question: ResMut<Question>,
    mut can_answer: ResMut<CanAnswer>,
    mut buttons: Query<(&mut BackgroundColor, &mut BorderColor), With<AnswerButton>>,
    mut answer_texts: Query<
        (Entity, &mut Text, &mut TextFont),
        (With<AnswerText>, Without<QuestionText>),
    >,
    questions: Res<Questions>,
    translation_direction: Res<TranslateDirection>,
    asset_server: Res<AssetServer>,
) {
    for _ in event_reader.read() {
        let mut thread_rng = rand::thread_rng();

        let (mut question_text, mut question_font) = question_text.single_mut();
        let new_question = questions
            .iter()
            .filter(|&q| q != &question.0)
            .choose(&mut thread_rng)
            .unwrap()
            .clone();
        **question_text = new_question.question(*translation_direction);
        question_font.font = translation_direction.question_font(&asset_server);
        question.0 = new_question;

        can_answer.0 = true;

        for (mut color, mut border_color) in &mut buttons {
            color.0 = NORMAL_BUTTON;
            border_color.0 = Color::BLACK;
        }

        let mut answer_text_entities = answer_texts.iter().map(|(e, ..)| e).collect::<Vec<_>>();
        answer_text_entities.sort();
        for (q, e) in questions.iter().zip(answer_text_entities) {
            if let Ok((_, mut text, mut font)) = answer_texts.get_mut(e) {
                **text = q.answer(*translation_direction);
                font.font = translation_direction.answer_font(&asset_server);
            }
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

    commands.spawn(Camera2d);
    let toplevel = commands
        .spawn((Node {
            display: Display::Flex,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },))
        .id();

    let top = commands
        .spawn((Node {
                flex_grow: 1.0,
                display: Display::Grid,
                width: Val::Percent(100.0),
                grid_template_rows: vec![RepeatedGridTrack::percent(1, 100.0)],
                grid_template_columns: vec![RepeatedGridTrack::percent(3, 100.0 / 3.0)],
                ..default()
            },
            BackgroundColor(Color::srgb(0.20, 0.20, 0.20)),
        ))
        .with_children(|commands| {
            commands
                .spawn(Node {
                        display: Display::Flex,
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Start,
                        align_items: AlignItems::Start,
                        padding: UiRect::all(Val::Px(5.0)),
                        ..default()
                })
                .with_children(|commands| {
                    commands
                        .spawn((
                            SettingsButton::SwitchDirection,
                            Button,
                         Node {
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                        ))
                        .with_children(|commands| {
                            commands.spawn((Text::new(
                                "ක -> ka",),
                                TextFont {
                                    font: asset_server.load(
                                        "fonts/Noto_Sans_Sinhala/NotoSansSinhala-VariableFont_wdth,wght.ttf",
                                    ),
                                    font_size: 50.0,
                                    ..default()
                                },
                            ));
                        });
                });

            commands.spawn(Node {
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
            })
            .with_children(|commands| {
                commands.spawn((
                    QuestionText,
                    Text::new( ""),
                    TextFont {
                        font: asset_server.load(
                            "fonts/Noto_Sans_Sinhala/NotoSansSinhala-VariableFont_wdth,wght.ttf",
                        ),
                        font_size: 75.0,
                        ..default()
                    },
                    TextLayout::new_with_justify(JustifyText::Center),
                ));
            });

            commands.spawn(Node {
                    display: Display::Flex,
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::FlexEnd,
                    padding: UiRect::all(Val::Px(5.0)),
                    ..default()
            }) .with_children(|commands| {
                commands
                    .spawn((
                        SettingsButton::RerollQuestions,
                        Button,
                            Node {
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                    ))
                    .with_children(|commands| {
                        commands.spawn((Text::new( " "),
                            TextFont {
                                font: asset_server.load(
                                    "fonts/0xProto/0xProtoNerdFont-Regular.ttf",
                                ),
                                font_size: 50.0,
                                ..default()
                            },
                        ));
                    });
            });
        })
        .id();

    let bottom = commands
        .spawn((
            AnswerBox,
            Node {
                display: Display::Grid,
                flex_grow: 3.0,
                width: Val::Percent(100.0),
                grid_template_rows: vec![RepeatedGridTrack::percent(5, 20.0)],
                grid_template_columns: vec![RepeatedGridTrack::percent(5, 20.0)],
                ..default()
            },
        ))
        .with_children(|commands| {
            for _ in &questions.0 {
                commands
                    .spawn((
                        AnswerButton,
                        Button,
                        Node {
                            //width: Val::Px(150.0),
                            //height: Val::Px(65.0),
                            border: UiRect::all(Val::Px(5.0)),
                            margin: UiRect::all(Val::Px(10.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BorderColor(Color::BLACK),
                        BackgroundColor(NORMAL_BUTTON),
                    ))
                    .with_children(|commands| {
                        commands.spawn((
                            AnswerText,
                            Text::new(""),
                            TextFont {
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
        (Changed<Interaction>, With<AnswerButton>),
    >,
    mut answered: EventWriter<AnsweredEvent>,
    can_answer: Res<CanAnswer>,
) {
    for (entity, interaction, mut color, mut border_color) in &mut interaction_query {
        if can_answer.0 {
            match *interaction {
                Interaction::Pressed => {
                    *color = PRESSED_BUTTON.into();
                    answered.send(AnsweredEvent(entity));
                }
                Interaction::Hovered => {
                    *color = HOVER_BUTTON.into();
                    border_color.0 = Color::WHITE;
                }
                Interaction::None => {
                    *color = NORMAL_BUTTON.into();
                    border_color.0 = Color::BLACK;
                }
            }
        }
    }
}

fn settings_button_system(
    mut interaction_query: Query<(&Interaction, &Children, &SettingsButton), Changed<Interaction>>,
    mut text: Query<(&mut Text, &mut TextColor)>,
    mut reroll_questions: EventWriter<RerollQuestionsEvent>,
    mut restart: EventWriter<RestartEvent>,
    mut translation_direction: ResMut<TranslateDirection>,
) {
    for (interaction, children, setting) in &mut interaction_query {
        let (mut text, mut color) = text.get_mut(children[0]).unwrap();
        **color = match *interaction {
            Interaction::Pressed => Color::srgb(1.0, 1.0, 1.0),
            Interaction::Hovered => Color::srgb(0.9, 0.9, 0.9),
            Interaction::None => Color::srgb(0.8, 0.8, 0.8),
        };
        if *interaction == Interaction::Pressed {
            match setting {
                SettingsButton::SwitchDirection => {
                    *translation_direction = match *translation_direction {
                        TranslateDirection::SinhalaToEnglish => {
                            **text = "ක <- ka".into();
                            TranslateDirection::EnglishToSinhala
                        }
                        TranslateDirection::EnglishToSinhala => {
                            **text = "ක -> ka".into();
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

        let answer = &text.get(children.get(answered_entity).unwrap()[0]).unwrap();
        let correct_answer = question.answer(*translation_direction);
        let correct_entity = buttons
            .iter()
            .find(|(_, _, _, children)| text.get(children[0]).unwrap().0 == correct_answer)
            .unwrap()
            .0;
        println!(
            "Question: {}, Answered: {}, correct answer: {correct_answer}",
            answer.0,
            question.question(*translation_direction)
        );

        for (entity, mut color, mut border_color, _) in &mut buttons {
            if entity == answered_entity {
                *color = PRESSED_BUTTON.into();
                if answer.0 == correct_answer {
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
