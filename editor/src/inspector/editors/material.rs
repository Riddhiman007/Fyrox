use crate::{Message, MessageDirection};
use fyrox::{
    asset::core::pool::Handle,
    core::parking_lot::Mutex,
    gui::{
        button::{ButtonBuilder, ButtonMessage},
        define_constructor,
        grid::{Column, GridBuilder, Row},
        inspector::{
            editors::{
                PropertyEditorBuildContext, PropertyEditorDefinition, PropertyEditorInstance,
                PropertyEditorMessageContext, PropertyEditorTranslationContext,
            },
            InspectorError, PropertyChanged,
        },
        message::UiMessage,
        text::{TextBuilder, TextMessage},
        widget::{Widget, WidgetBuilder},
        BuildContext, Control, Thickness, UiNode, UserInterface, VerticalAlignment,
    },
    material::SharedMaterial,
};
use std::{
    any::{Any, TypeId},
    fmt::{Debug, Formatter},
    ops::{Deref, DerefMut},
    sync::mpsc::Sender,
};

#[derive(Debug, Clone, PartialEq)]
pub enum MaterialFieldMessage {
    Material(SharedMaterial),
}

impl MaterialFieldMessage {
    define_constructor!(MaterialFieldMessage:Material => fn material(SharedMaterial), layout: false);
}

#[derive(Clone)]
pub struct MaterialFieldEditor {
    widget: Widget,
    sender: Sender<Message>,
    text: Handle<UiNode>,
    edit: Handle<UiNode>,
    material: SharedMaterial,
}

impl Debug for MaterialFieldEditor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MaterialFieldEditor")
    }
}

impl Deref for MaterialFieldEditor {
    type Target = Widget;

    fn deref(&self) -> &Self::Target {
        &self.widget
    }
}

impl DerefMut for MaterialFieldEditor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.widget
    }
}

impl Control for MaterialFieldEditor {
    fn query_component(&self, type_id: TypeId) -> Option<&dyn Any> {
        if type_id == TypeId::of::<Self>() {
            Some(self)
        } else {
            None
        }
    }

    fn handle_routed_message(&mut self, ui: &mut UserInterface, message: &mut UiMessage) {
        self.widget.handle_routed_message(ui, message);

        if message.destination() == self.edit {
            if let Some(ButtonMessage::Click) = message.data::<ButtonMessage>() {
                self.sender
                    .send(Message::OpenMaterialEditor(self.material.clone()))
                    .unwrap();
            }
        }

        if let Some(MaterialFieldMessage::Material(material)) = message.data() {
            if message.destination() == self.handle {
                self.material = material.clone();

                ui.send_message(TextMessage::text(
                    self.text,
                    MessageDirection::ToWidget,
                    make_name(&self.material),
                ));
            }
        }
    }
}

pub struct MaterialFieldEditorBuilder {
    widget_builder: WidgetBuilder,
}

fn make_name(material: &SharedMaterial) -> String {
    let name = material.lock().shader().data_ref().definition.name.clone();
    format!("{} - {} uses", name, material.use_count())
}

impl MaterialFieldEditorBuilder {
    pub fn new(widget_builder: WidgetBuilder) -> Self {
        Self { widget_builder }
    }

    pub fn build(
        self,
        ctx: &mut BuildContext,
        sender: Sender<Message>,
        material: SharedMaterial,
    ) -> Handle<UiNode> {
        let edit;
        let text;
        let editor = MaterialFieldEditor {
            widget: self
                .widget_builder
                .with_child(
                    GridBuilder::new(
                        WidgetBuilder::new()
                            .with_height(26.0)
                            .with_child({
                                text = TextBuilder::new(
                                    WidgetBuilder::new().with_margin(Thickness::uniform(1.0)),
                                )
                                .with_text(make_name(&material))
                                .with_vertical_text_alignment(VerticalAlignment::Center)
                                .build(ctx);
                                text
                            })
                            .with_child({
                                edit = ButtonBuilder::new(
                                    WidgetBuilder::new().with_width(32.0).on_column(1),
                                )
                                .with_text("...")
                                .build(ctx);
                                edit
                            }),
                    )
                    .add_row(Row::stretch())
                    .add_column(Column::stretch())
                    .add_column(Column::auto())
                    .build(ctx),
                )
                .build(),
            edit,
            sender,
            material,
            text,
        };

        ctx.add_node(UiNode::new(editor))
    }
}

#[derive(Debug)]
pub struct MaterialPropertyEditorDefinition {
    pub sender: Mutex<Sender<Message>>,
}

impl PropertyEditorDefinition for MaterialPropertyEditorDefinition {
    fn value_type_id(&self) -> TypeId {
        TypeId::of::<SharedMaterial>()
    }

    fn create_instance(
        &self,
        ctx: PropertyEditorBuildContext,
    ) -> Result<PropertyEditorInstance, InspectorError> {
        let value = ctx.property_info.cast_value::<SharedMaterial>()?;
        Ok(PropertyEditorInstance::Simple {
            editor: MaterialFieldEditorBuilder::new(WidgetBuilder::new()).build(
                ctx.build_context,
                self.sender.lock().clone(),
                value.clone(),
            ),
        })
    }

    fn create_message(
        &self,
        ctx: PropertyEditorMessageContext,
    ) -> Result<Option<UiMessage>, InspectorError> {
        let value = ctx.property_info.cast_value::<SharedMaterial>()?;
        Ok(Some(MaterialFieldMessage::material(
            ctx.instance,
            MessageDirection::ToWidget,
            value.clone(),
        )))
    }

    fn translate_message(&self, _ctx: PropertyEditorTranslationContext) -> Option<PropertyChanged> {
        None
    }
}
