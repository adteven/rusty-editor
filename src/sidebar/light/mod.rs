use crate::{
    gui::{BuildContext, Ui, UiMessage, UiNode},
    scene::{
        SceneCommand, SetLightCastShadowsCommand, SetLightColorCommand, SetLightScatterCommand,
        SetLightScatterEnabledCommand,
    },
    sidebar::{
        light::point::PointLightSection, light::spot::SpotLightSection, make_bool_input_field,
        make_text_mark, make_vec3_input_field, COLUMN_WIDTH, ROW_HEIGHT,
    },
    Message,
};
use rg3d::{
    core::pool::Handle,
    gui::{
        color::ColorFieldBuilder,
        grid::{Column, GridBuilder, Row},
        message::{
            CheckBoxMessage, ColorFieldMessage, MessageDirection, UiMessageData, Vec3EditorMessage,
            WidgetMessage,
        },
        widget::WidgetBuilder,
    },
    scene::node::Node,
};
use std::sync::mpsc::Sender;

mod point;
mod spot;

pub struct LightSection {
    pub section: Handle<UiNode>,
    color: Handle<UiNode>,
    cast_shadows: Handle<UiNode>,
    light_scatter: Handle<UiNode>,
    enable_scatter: Handle<UiNode>,
    pub point_light_section: PointLightSection,
    pub spot_light_section: SpotLightSection,
    sender: Sender<Message>,
}

impl LightSection {
    pub fn new(ctx: &mut BuildContext, sender: Sender<Message>) -> Self {
        let color;
        let cast_shadows;
        let light_scatter;
        let enable_scatter;
        let section = GridBuilder::new(
            WidgetBuilder::new()
                .with_child(make_text_mark(ctx, "Color", 0))
                .with_child({
                    color = ColorFieldBuilder::new(WidgetBuilder::new().on_column(1)).build(ctx);
                    color
                })
                .with_child(make_text_mark(ctx, "Cast Shadows", 1))
                .with_child({
                    cast_shadows = make_bool_input_field(ctx, 1);
                    cast_shadows
                })
                .with_child(make_text_mark(ctx, "Enable Scatter", 2))
                .with_child({
                    enable_scatter = make_bool_input_field(ctx, 2);
                    enable_scatter
                })
                .with_child(make_text_mark(ctx, "Scatter", 3))
                .with_child({
                    light_scatter = make_vec3_input_field(ctx, 3);
                    light_scatter
                }),
        )
        .add_column(Column::strict(COLUMN_WIDTH))
        .add_column(Column::stretch())
        .add_row(Row::strict(ROW_HEIGHT))
        .add_row(Row::strict(ROW_HEIGHT))
        .add_row(Row::strict(ROW_HEIGHT))
        .add_row(Row::strict(ROW_HEIGHT))
        .build(ctx);

        Self {
            section,
            color,
            cast_shadows,
            light_scatter,
            enable_scatter,
            point_light_section: PointLightSection::new(ctx, sender.clone()),
            spot_light_section: SpotLightSection::new(ctx, sender.clone()),
            sender,
        }
    }

    pub fn sync_to_model(&mut self, node: &Node, ui: &mut Ui) {
        if let Node::Light(light) = node {
            ui.send_message(Vec3EditorMessage::value(
                self.light_scatter,
                MessageDirection::ToWidget,
                light.scatter(),
            ));

            ui.send_message(ColorFieldMessage::color(
                self.color,
                MessageDirection::ToWidget,
                light.color(),
            ));

            ui.send_message(CheckBoxMessage::checked(
                self.cast_shadows,
                MessageDirection::ToWidget,
                Some(light.is_cast_shadows()),
            ));

            ui.send_message(CheckBoxMessage::checked(
                self.enable_scatter,
                MessageDirection::ToWidget,
                Some(light.is_scatter_enabled()),
            ));
        }
        ui.send_message(WidgetMessage::visibility(
            self.section,
            MessageDirection::ToWidget,
            node.is_light(),
        ));
        self.point_light_section.sync_to_model(node, ui);
        self.spot_light_section.sync_to_model(node, ui);
    }

    pub fn handle_message(&mut self, message: &UiMessage, node: &Node, handle: Handle<Node>) {
        if let Node::Light(light) = node {
            match &message.data() {
                UiMessageData::Vec3Editor(msg) => {
                    if let Vec3EditorMessage::Value(value) = *msg {
                        if message.destination() == self.light_scatter && light.scatter() != value {
                            self.sender
                                .send(Message::DoSceneCommand(SceneCommand::SetLightScatter(
                                    SetLightScatterCommand::new(handle, value),
                                )))
                                .unwrap();
                        }
                    }
                }
                UiMessageData::CheckBox(msg) => {
                    if let CheckBoxMessage::Check(value) = msg {
                        let value = value.unwrap_or(false);

                        if message.destination() == self.enable_scatter
                            && light.is_scatter_enabled() != value
                        {
                            self.sender
                                .send(Message::DoSceneCommand(
                                    SceneCommand::SetLightScatterEnabled(
                                        SetLightScatterEnabledCommand::new(handle, value),
                                    ),
                                ))
                                .unwrap();
                        } else if message.destination() == self.cast_shadows
                            && light.is_cast_shadows() != value
                        {
                            self.sender
                                .send(Message::DoSceneCommand(SceneCommand::SetLightCastShadows(
                                    SetLightCastShadowsCommand::new(handle, value),
                                )))
                                .unwrap();
                        }
                    }
                }
                UiMessageData::ColorField(msg) => {
                    if let ColorFieldMessage::Color(color) = *msg {
                        if message.destination() == self.color && light.color() != color {
                            self.sender
                                .send(Message::DoSceneCommand(SceneCommand::SetLightColor(
                                    SetLightColorCommand::new(handle, color),
                                )))
                                .unwrap();
                        }
                    }
                }
                _ => {}
            }
        }
        self.point_light_section
            .handle_message(message, node, handle);
        self.spot_light_section
            .handle_message(message, node, handle);
    }
}
