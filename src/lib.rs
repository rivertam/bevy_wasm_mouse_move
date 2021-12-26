mod html_body;

use bevy::input::mouse::{MouseButtonInput, MouseMotion};
use bevy::input::ElementState;
use bevy::prelude::*;
use gloo::events::{EventListener, EventListenerOptions};
use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::Arc;
use std::sync::Mutex;
use wasm_bindgen::JsCast;
use web_sys::{Event, MouseEvent, WheelEvent};

#[derive(Clone)]
struct MouseState {
    left_clicked: bool,
    right_clicked: bool,
    middle_clicked: bool,
}

impl MouseState {
    pub fn events(&self, previous: &MouseState) -> Vec<MouseButtonInput> {
        let mut events = vec![];

        if self.left_clicked != previous.left_clicked {
            events.push(MouseButtonInput {
                button: MouseButton::Left,
                state: if self.left_clicked {
                    ElementState::Pressed
                } else {
                    ElementState::Released
                },
            });
        }

        if self.right_clicked != previous.right_clicked {
            events.push(MouseButtonInput {
                button: MouseButton::Right,
                state: if self.right_clicked {
                    ElementState::Pressed
                } else {
                    ElementState::Released
                },
            });
        }

        if self.middle_clicked != previous.middle_clicked {
            events.push(MouseButtonInput {
                button: MouseButton::Middle,
                state: if self.middle_clicked {
                    ElementState::Pressed
                } else {
                    ElementState::Released
                },
            });
        }

        events
    }
}

pub struct BrowserMouse {
    x: Arc<AtomicI32>,
    y: Arc<AtomicI32>,

    delta_x: Arc<AtomicI32>,
    delta_y: Arc<AtomicI32>,

    scroll_delta: Arc<AtomicI32>,

    click_changes: Arc<Mutex<Vec<MouseButtonInput>>>,
}

impl BrowserMouse {
    pub fn new() -> Self {
        let delta_x = Arc::new(AtomicI32::new(0));
        let delta_y = Arc::new(AtomicI32::new(0));

        let scroll_delta = Arc::new(AtomicI32::new(0));

        let x = Arc::new(AtomicI32::new(0));
        let y = Arc::new(AtomicI32::new(0));

        let state = Arc::new(Mutex::new(MouseState {
            left_clicked: false,
            right_clicked: false,
            middle_clicked: false,
        }));

        let click_changes = Arc::new(Mutex::new(vec![]));

        EventListener::new(&html_body::get(), "mousemove", {
            let delta_x = Arc::clone(&delta_x);
            let delta_y = Arc::clone(&delta_y);

            let x = Arc::clone(&x);
            let y = Arc::clone(&y);

            move |event: &Event| {
                let mouse_event = event.clone().dyn_into::<MouseEvent>().unwrap();
                x.store(mouse_event.client_x(), SeqCst);
                y.store(mouse_event.client_y(), SeqCst);
                delta_x.fetch_add(mouse_event.movement_x(), SeqCst);
                delta_y.fetch_add(mouse_event.movement_y(), SeqCst);
            }
        })
        .forget();

        EventListener::new_with_options(
            &html_body::get(),
            "wheel",
            EventListenerOptions::enable_prevent_default(),
            {
                let scroll_delta = Arc::clone(&scroll_delta);

                move |event: &Event| {
                    let wheel_event = event.clone().dyn_into::<WheelEvent>().unwrap();
                    scroll_delta.fetch_add(wheel_event.delta_y() as i32, SeqCst);
                }
            },
        )
        .forget();

        let click_handler = || {
            let state = Arc::clone(&state);
            let click_changes = Arc::clone(&click_changes);

            move |event: &Event| {
                let event = event.clone().dyn_into::<MouseEvent>().unwrap();
                let new_state = MouseState {
                    left_clicked: event.buttons() & 1 != 0,
                    right_clicked: event.buttons() & 2 != 0,
                    middle_clicked: event.buttons() & 4 != 0,
                };

                let mut old_state = state.lock().expect("could not lock mouse state");
                let mut changes = click_changes
                    .lock()
                    .expect("could not lock mouse click changes");

                changes.append(&mut new_state.events(&*old_state));

                *old_state = new_state.clone();
            }
        };

        EventListener::new(&html_body::get(), "mousedown", click_handler()).forget();

        EventListener::new(&html_body::get(), "mouseup", click_handler()).forget();

        Self {
            x,
            y,
            delta_x,
            delta_y,
            click_changes,
        }
    }

    pub fn get_delta_and_reset(&self) -> Vec2 {
        let delta = Vec2::new(
            self.delta_x.load(SeqCst) as f32,
            self.delta_y.load(SeqCst) as f32,
        );
        self.delta_x.store(0, SeqCst);
        self.delta_y.store(0, SeqCst);
        delta
    }

    fn get_changes_and_reset(&self) -> Vec<MouseButtonInput> {
        let mut changes = self
            .click_changes
            .lock()
            .expect("failed to lock mouse clicks");

        let cloned: Vec<_> = changes.clone();

        changes.clear();

        cloned
    }
}

pub fn emit_mouse_events(
    mut mouse_motion_events: EventWriter<MouseMotion>,
    mut mouse_button_events: EventWriter<MouseButtonInput>,
    mut cursor_moved_events: EventWriter<CursorMoved>,
    windows: ResMut<Windows>,
    mouse: Res<BrowserMouse>,
) {
    let delta = mouse.get_delta_and_reset();
    if delta != Vec2::ZERO {
        mouse_motion_events.send(MouseMotion { delta });

        cursor_moved_events.send(CursorMoved {
            id: windows.get_primary().expect("no primary window found").id(),
            position: Vec2::new(mouse.x.load(SeqCst) as f32, mouse.y.load(SeqCst) as f32),
        })
    }

    let click_changes = mouse.get_changes_and_reset();
    for change in click_changes {
        mouse_button_events.send(change);
    }
}

pub struct PatchMouseMotionPlugin;

impl Plugin for PatchMouseMotionPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(emit_mouse_events.system())
            .insert_resource(BrowserMouse::new());
    }
}
