use std::sync::Arc;

use bevy::ecs::spawn::SpawnIter;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::text::ComputedTextBlock;
use bevy::text::CosmicFontSystem;
use bevy::text::LineHeight;
use bevy::text::cosmic_text;
use bevy::text::cosmic_text::fontdb;
use bevy::text::cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping, Wrap};
use bevy::text::TextBounds;
use bevy::text::TextLayoutInfo;
use bevy::ui::widget::TextNodeFlags;
use bevy::window::PrimaryWindow;
use bevy_asset_loader::prelude::*;
//use bevy_inspector_egui::bevy_egui::EguiPlugin;
use rand::{prelude::SliceRandom, seq::IteratorRandom};

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVER_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.20, 0.20, 0.20);

#[derive(Component)]
struct FitToParent;

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
    NextDictionary,
}

#[derive(AssetCollection, Resource)]
struct Fonts {
    #[asset(path = "fonts/Noto_Sans_Sinhala/NotoSansSinhala-VariableFont_wdth,wght.subset.ttf")]
    sinhala: Handle<Font>,
    #[asset(path = "fonts/Noto_Serif/NotoSerif-VariableFont_wdth,wght.subset.ttf")]
    english: Handle<Font>,
    #[asset(path = "fonts/0xProto/0xProtoNerdFont-Regular.subset.ttf")]
    icons: Handle<Font>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum LoadingStates {
    #[default]
    AssetLoading,
    Loaded,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum AnsweringState {
    #[default]
    Answering,
    Answered(u32),
}

#[derive(Debug, Resource, Clone, Copy)]
enum TranslateDirection {
    SinhalaToEnglish,
    EnglishToSinhala,
}

impl TranslateDirection {
    pub fn question_font(&self, fonts: &Fonts) -> Handle<Font> {
        match *self {
            TranslateDirection::SinhalaToEnglish => fonts.sinhala.clone(),
            TranslateDirection::EnglishToSinhala => fonts.english.clone(),
        }
    }

    pub fn answer_font(&self, fonts: &Fonts) -> Handle<Font> {
        match *self {
            TranslateDirection::SinhalaToEnglish => fonts.english.clone(),
            TranslateDirection::EnglishToSinhala => fonts.sinhala.clone(),
        }
    }
}

#[derive(Debug, Resource, Deref, DerefMut, PartialEq, Eq)]
struct Question(Pair);
#[derive(Debug, Resource, Deref, DerefMut)]
struct Questions(Vec<Pair>);
#[derive(Debug, Resource, Deref, DerefMut)]
struct AllQuestions(Vec<Dictionary>);

#[derive(Event)]
struct AnsweredEvent(pub Entity);
#[derive(Event)]
struct RestartEvent;
#[derive(Event)]
struct NewQuestion;
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

    impl<'a> TryFrom<Vec<&'a str>> for Pair {
        type Error = Vec<&'a str>;
        fn try_from(words: Vec<&'a str>) -> Result<Self, Self::Error> {
            let [sinhala, english] = words.try_into()?;
            Ok(Self {
                sinhala: sinhala.into(),
                english: english.into(),
            })
        }
    }
}

#[derive(Resource, Debug, Clone)]
struct Dictionary {
    title: String,
    entries: Vec<Pair>,
}

#[derive(Resource, Debug, Clone)]
struct CurrentDictionary(usize);

fn main() {
    let dictionaries: Vec<Dictionary> = include_str!("../dictionary.txt")
        .split("\n\n")
        .filter_map(|dictionary| {
            let mut lines = dictionary.lines();
            let title = lines.next().unwrap().trim_end_matches(':').to_string();
            if title == "icons" {
                return None;
            }
            let entries = lines
                .map(|l| {
                    l.split(" - ")
                        .map(|t| t.trim())
                        .collect::<Vec<_>>()
                        .try_into()
                        .unwrap()
                })
                .collect();
            Some(Dictionary { title, entries })
        })
        .collect();

    let mut thread_rng = rand::thread_rng();
    let questions = dictionaries[0]
        .entries
        .iter()
        .take(25)
        .cloned()
        .collect::<Vec<_>>();
    let question = questions.iter().choose(&mut thread_rng).unwrap().clone();

    App::new()
        .add_event::<AnsweredEvent>()
        .add_event::<RestartEvent>()
        .add_event::<RerollQuestionsEvent>()
        .insert_resource(Question(question))
        .insert_resource(Questions(questions))
        .insert_resource(AllQuestions(dictionaries))
        .insert_resource(CurrentDictionary(0))
        .insert_resource(TranslateDirection::SinhalaToEnglish)
        .init_resource::<FitToParentFonts>()
        .add_plugins((
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
            //EguiPlugin {
            //    enable_multipass_for_primary_context: true,
            //},
            //WorldInspectorPlugin::new(),
            //SimpleSubsecondPlugin::default(),
        ))
        .init_state::<AnsweringState>()
        .init_state::<LoadingStates>()
        .add_loading_state(
            LoadingState::new(LoadingStates::AssetLoading)
                .continue_to_state(LoadingStates::Loaded)
                .load_collection::<Fonts>(),
        )
        .add_systems(OnEnter(LoadingStates::Loaded), spawn_text)
        .add_systems(
            Update,
            (
                settings_button_system,
                reset_one_second_after_answer,
                button_system,
                handle_answer,
            )
                .chain()
                .run_if(in_state(LoadingStates::Loaded)),
        )
        .add_systems(Update, fit_to_parent) //.run_if(on_timer(Duration::from_millis(100))))
        .add_observer(setup_question)
        .add_observer(reroll_questions)
        .add_observer(next_question)
        .run();
}

fn buffer_dimensions(buffer: &Buffer) -> Vec2 {
    let (width, height) = buffer
        .layout_runs()
        .map(|run| (run.line_w, run.line_height))
        .reduce(|(w1, h1), (w2, h2)| (w1.max(w2), h1 + h2))
        .unwrap_or((0.0, 0.0));

    Vec2::new(width, height).ceil()
}

#[derive(Clone)]
struct FontFaceInfo {
    stretch: fontdb::Stretch,
    style: fontdb::Style,
    weight: fontdb::Weight,
    family_name: Arc<str>,
}

/// Find the size of the text when rendered with the given parameters.
pub fn measure_text(
    fonts: &Assets<Font>,
    font_system: &mut FontSystem,
    scale_factor: f32,
    line_height: LineHeight,
    alignment: JustifyText,
    width: Option<f32>,
    height: Option<f32>,
    linebreak: LineBreak,
    entity: Entity,
    text_reader: &mut TextUiReader,
    font_size_override: f32,
    map_handle_to_font_id: &mut HashMap<AssetId<Font>, (fontdb::ID, Arc<str>)>,
) -> Vec2 {
    let mut buffer = Buffer::new(
        font_system,
        Metrics {
            font_size: font_size_override,
            line_height: match line_height {
                LineHeight::Px(px) => px,
                LineHeight::RelativeToFont(scale) => scale * font_size_override,
            },
        }
        .scale(scale_factor),
    );
    buffer.set_size(font_system, width, height);
    buffer.set_wrap(
        font_system,
        match linebreak {
            LineBreak::WordBoundary => Wrap::Word,
            LineBreak::AnyCharacter => Wrap::Glyph,
            LineBreak::WordOrCharacter => Wrap::WordOrGlyph,
            LineBreak::NoWrap => Wrap::None,
        },
    );
    let mut spans: Vec<(usize, &str, TextFont, FontFaceInfo, Color)> = vec![];

    fn load_font_to_fontdb(
        font_handle: Handle<Font>,
        font_system: &mut FontSystem,
        map_handle_to_font_id: &mut HashMap<AssetId<Font>, (fontdb::ID, Arc<str>)>,
        fonts: &Assets<Font>,
    ) -> FontFaceInfo {
        let (face_id, family_name) = map_handle_to_font_id
            .entry(font_handle.id())
            .or_insert_with(|| {
                let font = fonts.get(font_handle.id()).expect(
                "Tried getting a font that was not available, probably due to not being loaded yet",
            );
                let data = Arc::clone(&font.data);
                let ids = font_system
                    .db_mut()
                    .load_font_source(fontdb::Source::Binary(data));

                let face_id = *ids.last().unwrap();
                let face = font_system.db().face(face_id).unwrap();
                let family_name = Arc::from(face.families[0].0.as_str());

                (face_id, family_name)
            });
        let face = font_system.db().face(*face_id).unwrap();

        FontFaceInfo {
            stretch: face.stretch,
            style: face.style,
            weight: face.weight,
            family_name: family_name.clone(),
        }
    }

    for (span_index, (entity, _depth, span, text_font, color)) in
        text_reader.iter(entity).enumerate()
    {
        let mut text_font = text_font.clone();
        if span.is_empty() {
            continue;
        }
        // Return early if a font is not loaded yet.
        if !fonts.contains(text_font.font.id()) {
            spans.clear();

            panic!();
        }

        // Get max font size for use in cosmic Metrics.
        text_font.font_size = font_size_override;

        // Load Bevy fonts into cosmic-text's font system.
        let face_info = load_font_to_fontdb(
            text_font.font.clone(),
            font_system,
            map_handle_to_font_id,
            fonts,
        );

        // Save spans that aren't zero-sized.
        if scale_factor <= 0.0 || font_size_override <= 0.0 {
            once!(warn!(
                "Text span {entity} has a font size <= 0.0. Nothing will be displayed.",
            ));

            continue;
        }
        spans.push((span_index, span, text_font, face_info, color));
    }
    fn get_attrs<'a>(
        span_index: usize,
        text_font: &TextFont,
        color: Color,
        face_info: &'a FontFaceInfo,
        scale_factor: f64,
    ) -> Attrs<'a> {
        Attrs::new()
            .metadata(span_index)
            .family(Family::Name(&face_info.family_name))
            .stretch(face_info.stretch)
            .style(face_info.style)
            .weight(face_info.weight)
            .metrics(
                Metrics {
                    font_size: text_font.font_size,
                    line_height: match text_font.line_height {
                        LineHeight::Px(px) => px,
                        LineHeight::RelativeToFont(scale) => scale * text_font.font_size,
                    },
                }
                .scale(scale_factor as f32),
            )
            .color(cosmic_text::Color(color.to_linear().as_u32()))
    }
    let spans_iter = spans
        .iter()
        .map(|(span_index, span, text_font, font_info, color)| {
            (
                *span,
                get_attrs(
                    *span_index,
                    text_font,
                    *color,
                    font_info,
                    scale_factor as f64,
                ),
            )
        });
    buffer.set_rich_text(
        font_system,
        spans_iter,
        Attrs::new(),
        Shaping::Advanced,
        Some(alignment.into()),
    );
    buffer.shape_until_scroll(font_system, false);
    buffer.set_size(font_system, width, height);
    let (width, height) = buffer
        .layout_runs()
        .map(|run| (run.line_w, run.line_height))
        .reduce(|(w1, h1), (w2, h2)| (w1.max(w2), h1 + h2))
        .unwrap_or((0.0, 0.0));
    (Vec2::new(width, height)).ceil()
}

#[derive(Debug, Resource, Default)]
struct FitToParentFonts(HashMap<AssetId<Font>, (fontdb::ID, Arc<str>)>);

fn fit_to_parent(
    mut font_system: ResMut<CosmicFontSystem>,
    fonts: Res<Assets<Font>>,
    mut texts: Query<
        (
            Entity,
            &ChildOf,
            Ref<Text>,
            &mut TextFont,
            Ref<ComputedTextBlock>,
            &TextLayout,
            &ComputedNode,
            &TextLayoutInfo,
            &TextNodeFlags,
        ),
        With<FitToParent>,
    >,
    nodes: Query<Ref<ComputedNode>>,
    //mut param_set: ParamSet<(Query<&mut TextFont>, TextUiReader)>,
    mut map_handle_to_font_id: ResMut<FitToParentFonts>,
) -> Result {
    for (entity, parent, text, mut text_font, computed_text_block, text_layout, computed_node, text_layout_info, text_node_flags) in &mut texts {
        let parent_node = nodes.get(parent.parent())?;

        let parent_size = 0.9 * parent_node.content_size();
        let computed_size = computed_node.content_size();
        if computed_size.x > parent_size.x || computed_size.y > parent_size.y {
            text_font.font_size -= 5.0;
        }
        //let content_size = buffer_dimensions(computed_text_block.buffer());
        //dbg!(text_layout_info);

        //for font_size in (1..10).map(|i| i as f32 * 10.0).rev() {
        //    let text_font = param_set.p0().get(entity)?.clone();
        //    let mut text_reader = param_set.p1();
        //    let measurement = measure_text(
        //        &fonts,
        //        &mut font_system.0,
        //        computed_node.inverse_scale_factor().recip(),
        //        text_font.line_height,
        //        JustifyText::Center,
        //        Some(computed_size.x),
        //        Some(computed_size.y),
        //        text_layout.linebreak,
        //        entity,
        //        &mut text_reader,
        //        font_size,
        //        &mut map_handle_to_font_id.0,
        //    );
        //    let fits = measurement.x <= computed_size.x && measurement.y <= computed_size.y;
        //    let content_size = text_layout_info.size * computed_node.inverse_scale_factor().recip();
        //    println!(
        //        "{}, at font_size {font_size} fits? {fits}. measurement: {measurement}, content_size: {content_size}, parent size: {computed_size}",
        //        text.0,
        //    );
        //    if fits && measurement.x != 0. && measurement.y != 0. {
        //        param_set.p0().get_mut(entity)?.font_size = font_size;
        //        break;
        //    }
        //}
    }

    Ok(())
}

fn reset_one_second_after_answer(
    time: Res<Time>,
    answering_state: Res<State<AnsweringState>>,
    mut commands: Commands,
) {
    if let AnsweringState::Answered(start_time) = **answering_state
        && time.elapsed_secs() * 1000.0 > start_time as f32 + 1000.0
    {
        commands.trigger(NewQuestion);
    }
}

fn reroll_questions(
    _: Trigger<RerollQuestionsEvent>,
    mut commands: Commands,
    all_questions: Res<AllQuestions>,
    current_dictionary: Res<CurrentDictionary>,
    mut questions: ResMut<Questions>,
) {
    let mut thread_rng = rand::thread_rng();
    questions.0 = all_questions[current_dictionary.0]
        .entries
        .iter()
        .cloned()
        .choose_multiple(&mut thread_rng, 25);
    questions.0.shuffle(&mut thread_rng);

    commands.trigger(NewQuestion);
}

fn next_question(
    _: Trigger<NewQuestion>,
    mut commands: Commands,
    questions: Res<Questions>,
    mut question: ResMut<Question>,
) {
    let mut thread_rng = rand::thread_rng();
    let new_question = questions
        .iter()
        .filter(|&q| q != &question.0)
        .choose(&mut thread_rng)
        .unwrap()
        .clone();
    question.0 = new_question;

    commands.trigger(RestartEvent);
}

fn setup_question(
    _: Trigger<RestartEvent>,
    mut question_text: Query<(&mut Text, &mut TextFont), (With<QuestionText>, Without<AnswerText>)>,
    mut next_answering_state: ResMut<NextState<AnsweringState>>,
    mut buttons: Query<(&mut BackgroundColor, &mut BorderColor), With<AnswerButton>>,
    mut answer_texts: Query<
        (Entity, &mut Text, &mut TextFont),
        (With<AnswerText>, Without<QuestionText>),
    >,
    questions: Res<Questions>,
    question: Res<Question>,
    translation_direction: Res<TranslateDirection>,
    fonts: Res<Fonts>,
) {
    let (mut question_text, mut question_font) = question_text.single_mut().unwrap();
    **question_text = question.question(*translation_direction);
    question_font.font = translation_direction.question_font(&fonts);
    question_font.font_size = 75.0;

    next_answering_state.set(AnsweringState::Answering);

    for (mut color, mut border_color) in &mut buttons {
        color.0 = NORMAL_BUTTON;
        border_color.0 = Color::BLACK;
    }

    let mut answer_text_entities = answer_texts.iter().map(|(e, ..)| e).collect::<Vec<_>>();
    answer_text_entities.sort();
    for (q, e) in questions.iter().zip(answer_text_entities) {
        if let Ok((_, mut text, mut font)) = answer_texts.get_mut(e) {
            **text = q.answer(*translation_direction);
            font.font = translation_direction.answer_font(&fonts);
            font.font_size = 50.0;
        }
    }
}

fn button() -> impl Bundle {
    (
        Button,
        Node {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
    )
}

fn text_with_font(text: impl Into<String>, font_size: f32, font: Handle<Font>) -> impl Bundle {
    (
        Text::new(text),
        TextFont::from_font(font).with_font_size(font_size),
        TextLayout::new_with_justify(JustifyText::Center),
    )
}

fn sinhala(text: impl Into<String>, font_size: f32, fonts: &Res<Fonts>) -> impl Bundle {
    text_with_font(text, font_size, fonts.sinhala.clone())
}

fn english(text: impl Into<String>, font_size: f32, fonts: &Res<Fonts>) -> impl Bundle {
    text_with_font(text, font_size, fonts.english.clone())
}

fn icon(text: impl Into<String>, font_size: f32, fonts: &Res<Fonts>) -> impl Bundle {
    text_with_font(text, font_size, fonts.icons.clone())
}

fn top(dictionary_title: &str, fonts: &Res<Fonts>) -> impl Bundle {
    (
        Node {
            display: Display::Grid,
            width: Val::Percent(100.0),
            grid_template_rows: vec![RepeatedGridTrack::percent(1, 100.0)],
            grid_template_columns: vec![RepeatedGridTrack::percent(3, 100.0 / 3.0)],
            ..default()
        },
        BackgroundColor(Color::srgb(0.20, 0.20, 0.20)),
        children![
            (
                Node {
                    display: Display::Flex,
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Start,
                    align_items: AlignItems::Start,
                    padding: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
                children![(
                    SettingsButton::SwitchDirection,
                    button(),
                    children![sinhala("ක -> ka", 50.0, fonts)],
                )],
            ),
            (
                Node {
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                children![(QuestionText, FitToParent, sinhala("", 75.0, fonts))],
            ),
            (
                Node {
                    display: Display::Flex,
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::FlexEnd,
                    padding: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
                children![
                    (
                        SettingsButton::RerollQuestions,
                        button(),
                        children![icon(" ", 50.0, fonts)],
                    ),
                    (
                        SettingsButton::NextDictionary,
                        button(),
                        children![english(dictionary_title, 50.0, fonts)],
                    )
                ],
            )
        ],
    )
}

fn bottom(questions: &Res<Questions>, fonts: &Res<Fonts>) -> impl Bundle {
    (
        AnswerBox,
        Node {
            display: Display::Grid,
            width: Val::Percent(100.0),
            grid_template_rows: vec![RepeatedGridTrack::percent(5, 20.0)],
            grid_template_columns: vec![RepeatedGridTrack::percent(5, 20.0)],
            ..default()
        },
        Children::spawn(SpawnIter(
            questions
                .0
                .iter()
                .map(|_| {
                    (
                        AnswerButton,
                        Button,
                        Node {
                            border: UiRect::all(Val::Px(5.0)),
                            margin: UiRect::all(Val::Px(10.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BorderColor(Color::BLACK),
                        BackgroundColor(NORMAL_BUTTON),
                        children![(AnswerText, FitToParent, english("", 50.0, fonts))],
                    )
                })
                .collect::<Vec<_>>()
                .into_iter(),
        )),
    )
}

fn spawn_text(
    mut commands: Commands,
    fonts: Res<Fonts>,
    questions: Res<Questions>,
    current_dictionary: Res<CurrentDictionary>,
    dictionaries: Res<AllQuestions>,
) {
    commands.spawn(Camera2d);

    commands.spawn((
        Node {
            display: Display::Grid,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            grid_template_rows: vec![
                RepeatedGridTrack::percent(1, 20.0),
                RepeatedGridTrack::percent(1, 80.0),
            ],
            grid_template_columns: vec![RepeatedGridTrack::percent(1, 100.0)],
            ..default()
        },
        children![
            top(&dictionaries[current_dictionary.0].title, &fonts),
            bottom(&questions, &fonts)
        ],
    ));

    commands.trigger(RestartEvent);
}

fn button_system(
    mut interaction_query: Query<
        (Entity, &Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<AnswerButton>),
    >,
    mut answered: EventWriter<AnsweredEvent>,
    answering_state: Res<State<AnsweringState>>,
) {
    if !matches!(**answering_state, AnsweringState::Answering) {
        return;
    }
    for (entity, interaction, mut color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                answered.write(AnsweredEvent(entity));
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

fn settings_button_system(
    mut interaction_query: Query<(&Interaction, &Children, &SettingsButton), Changed<Interaction>>,
    mut text: Query<(&mut Text, &mut TextColor)>,
    mut commands: Commands,
    mut translation_direction: ResMut<TranslateDirection>,
    mut current_dictionary: ResMut<CurrentDictionary>,
    dictionaries: Res<AllQuestions>,
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
                    commands.trigger(RestartEvent);
                }
                SettingsButton::RerollQuestions => {
                    commands.trigger(RerollQuestionsEvent);
                }
                SettingsButton::NextDictionary => {
                    current_dictionary.0 = (current_dictionary.0 + 1) % dictionaries.len();
                    **text = dictionaries[current_dictionary.0].title.clone();
                    commands.trigger(RerollQuestionsEvent);
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
    mut next_answering_state: ResMut<NextState<AnsweringState>>,
    question: Res<Question>,
    translation_direction: Res<TranslateDirection>,
    time: Res<Time>,
) {
    for AnsweredEvent(answered_entity) in answered.drain().take(1) {
        next_answering_state.set(AnsweringState::Answered(
            (time.elapsed_secs() * 1000.0) as u32,
        ));

        let answer = &text.get(children.get(answered_entity).unwrap()[0]).unwrap();
        let correct_answer = question.answer(*translation_direction);
        let correct_entity = buttons
            .iter()
            .find(|(_, _, _, children)| text.get(children[0]).unwrap().0 == correct_answer)
            .unwrap()
            .0;
        println!(
            "Question: {}, Answered: {}, correct answer: {correct_answer}",
            question.question(*translation_direction),
            answer.0,
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
