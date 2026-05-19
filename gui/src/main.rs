use iced::widget::{column, row, text, container, button, Space, image};
use iced::advanced::widget::{self, Widget};
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::mouse;
use iced::{Alignment, Element, Length, Sandbox, Settings, Theme, Color, Border, Shadow, Vector, theme, ContentFit, Rectangle, Size, Event};

pub fn main() -> iced::Result {
    ArcGui::run(Settings::default())
}

struct ArcGui {
    pinned_apps: Vec<String>,
}

#[derive(Debug, Clone)]
enum Message {
    EntryClicked,
    AppClicked(String),
}

// Custom Stack Widget to layer widgets on top of each other in iced 0.12
struct Stack<'a, Message> {
    children: Vec<Element<'a, Message, Theme, iced::Renderer>>,
}

impl<'a, Message> Stack<'a, Message> {
    pub fn new() -> Self {
        Self { children: Vec::new() }
    }

    pub fn push(mut self, child: impl Into<Element<'a, Message, Theme, iced::Renderer>>) -> Self {
        self.children.push(child.into());
        self
    }
}

impl<'a, Message> Widget<Message, Theme, iced::Renderer> for Stack<'a, Message> {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(
        &self,
        tree: &mut widget::Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = limits.resolve(Length::Fill, Length::Fill, Size::ZERO);
        let child_limits = layout::Limits::new(Size::ZERO, size);
        
        let nodes = self.children.iter()
            .zip(&mut tree.children)
            .map(|(child, child_tree)| {
                child.as_widget().layout(child_tree, renderer, &child_limits)
            })
            .collect::<Vec<_>>();
            
        layout::Node::with_children(size, nodes)
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        for ((child, child_tree), child_layout) in self.children.iter().zip(&tree.children).zip(layout.children()) {
            child.as_widget().draw(child_tree, renderer, theme, style, child_layout, cursor, viewport);
        }
    }

    fn children(&self) -> Vec<widget::Tree> {
        self.children.iter().map(|child| widget::Tree::new(child)).collect()
    }

    fn diff(&self, tree: &mut widget::Tree) {
        tree.diff_children(&self.children);
    }
    
    fn on_event(
        &mut self,
        tree: &mut widget::Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> iced::event::Status {
        let mut status = iced::event::Status::Ignored;
        let children_layouts: Vec<Layout<'_>> = layout.children().collect();
        
        for ((child, child_tree), child_layout) in self.children.iter_mut().zip(&mut tree.children).zip(children_layouts).rev() {
            let child_status = child.as_widget_mut().on_event(
                child_tree,
                event.clone(),
                child_layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
            if child_status == iced::event::Status::Captured {
                status = child_status;
                break;
            }
        }
        status
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        let children_layouts: Vec<Layout<'_>> = layout.children().collect();
        
        self.children.iter().zip(&tree.children).zip(children_layouts).rev().find_map(|((child, child_tree), child_layout)| {
            let interaction = child.as_widget().mouse_interaction(child_tree, child_layout, cursor, viewport, renderer);
            if interaction != mouse::Interaction::Idle {
                Some(interaction)
            } else {
                None
            }
        }).unwrap_or(mouse::Interaction::Idle)
    }
}

impl<'a, Message> From<Stack<'a, Message>> for Element<'a, Message, Theme, iced::Renderer>
where
    Message: 'a,
{
    fn from(stack: Stack<'a, Message>) -> Self {
        Element::new(stack)
    }
}

// Custom StyleSheet for the translucent glassmorphic Dock container
struct DockStyle;

impl container::StyleSheet for DockStyle {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Color::from_rgba8(25, 25, 35, 0.55).into()), // Semi-transparent glass background
            border: Border {
                color: Color::from_rgba8(255, 255, 255, 0.18),
                width: 1.0,
                radius: 24.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba8(0, 0, 0, 0.4),
                offset: Vector::new(0.0, 8.0),
                blur_radius: 20.0,
            },
            text_color: None,
        }
    }
}

// Custom StyleSheet for standard dock app buttons (macOS style: floating icon, subtle hover overlay)
struct DockButtonStyle;

impl button::StyleSheet for DockButtonStyle {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: None, // Floating icon
            text_color: Color::WHITE,
            border: Border::default(),
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Color::from_rgba8(255, 255, 255, 0.15).into()), // Elegant light container on hover
            text_color: Color::WHITE,
            border: Border {
                color: Color::from_rgba8(255, 255, 255, 0.15),
                width: 1.0,
                radius: 14.0.into(),
            },
            shadow_offset: Vector::new(0.0, 2.0),
            ..Default::default()
        }
    }

    fn pressed(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Color::from_rgba8(255, 255, 255, 0.08).into()),
            text_color: Color::WHITE,
            border: Border {
                color: Color::from_rgba8(255, 255, 255, 0.05),
                width: 1.0,
                radius: 14.0.into(),
            },
            ..Default::default()
        }
    }
}

impl Sandbox for ArcGui {
    type Message = Message;

    fn new() -> Self {
        Self {
            pinned_apps: vec![
                "Terminal".to_string(),
                "Files".to_string(),
                "Browser".to_string(),
                "AI Agent".to_string(),
            ],
        }
    }

    fn title(&self) -> String {
        String::from("Arc OS")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::EntryClicked => println!("Entry portal opened"),
            Message::AppClicked(app) => println!("Launching {}...", app),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // Base Layer: Wallpaper Image (Embedded at compile-time)
        let background_bytes = include_bytes!("../assets/images/background.jpeg");
        let background_handle = image::Handle::from_memory(background_bytes.as_slice());
        let background = image(background_handle)
            .content_fit(ContentFit::Cover)
            .width(Length::Fill)
            .height(Length::Fill);

        // Foreground Layout: Center Text and Bottom Dock
        let mut dock_items = row![].spacing(16).align_items(Alignment::Center);

        // The "Entry" Button using macOS System Preferences icon
        let entry_bytes = include_bytes!("../assets/icons/SystemPreferences.png");
        let entry_handle = image::Handle::from_memory(entry_bytes.as_slice());
        dock_items = dock_items.push(
            button(
                image(entry_handle)
                    .width(48)
                    .height(48)
            )
            .on_press(Message::EntryClicked)
            .style(theme::Button::Custom(Box::new(DockButtonStyle))),
        );

        // Space / Divider line
        dock_items = dock_items.push(
            container(Space::with_width(1))
                .height(32)
                .style(theme::Container::Custom(Box::new(DockStyle))), // reuse dock border color
        );

        // Pinned Apps
        for app in &self.pinned_apps {
            let icon_bytes: &[u8] = match app.as_str() {
                "Terminal" => include_bytes!("../assets/icons/Terminal.png"),
                "Files" => include_bytes!("../assets/icons/Preview.png"),
                "Browser" => include_bytes!("../assets/icons/Safari.png"),
                "AI Agent" => include_bytes!("../assets/icons/Automator.png"),
                _ => include_bytes!("../assets/icons/Console.png"),
            };
            
            let icon_handle = image::Handle::from_memory(icon_bytes);

            dock_items = dock_items.push(
                button(
                    image(icon_handle)
                        .width(48)
                        .height(48)
                )
                .on_press(Message::AppClicked(app.clone()))
                .style(theme::Button::Custom(Box::new(DockButtonStyle))),
            );
        }

        let dock = container(dock_items)
            .padding(10)
            .style(theme::Container::Custom(Box::new(DockStyle)));

        let overlay = column![
            // Top spacer
            Space::with_height(Length::Fill),

            // Subtle brand name
            container(
                column![
                    text("ARC OS")
                        .size(54)
                        .style(Color::from_rgba8(255, 255, 255, 0.9)),
                    Space::with_height(4),
                    text("AI-Native Operating System")
                        .size(16)
                        .style(Color::from_rgba8(255, 255, 255, 0.6)),
                ]
                .align_items(Alignment::Center)
            )
            .width(Length::Fill)
            .center_x(),

            // Large spacer to push dock down
            Space::with_height(Length::Fill),

            // Bottom Dock
            container(dock)
                .width(Length::Fill)
                .padding(24)
                .center_x()
        ]
        .width(Length::Fill)
        .height(Length::Fill);

        let stack = Stack::new()
            .push(background)
            .push(overlay);

        stack.into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}
