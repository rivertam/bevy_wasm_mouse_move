mod html_body;

use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use gloo::events::EventListener;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::Arc;
use wasm_bindgen::JsCast;
use web_sys::{Event, MouseEvent};

pub struct BrowserMouse {
    delta_x: Arc<AtomicI32>,
    delta_y: Arc<AtomicI32>,
}

impl BrowserMouse {
    pub fn new() -> Self {
        let delta_x = Arc::new(AtomicI32::new(0));
        let delta_y = Arc::new(AtomicI32::new(0));

        EventListener::new(&html_body::get(), "mousemove", {
            let delta_x = Arc::clone(&delta_x);
            let delta_y = Arc::clone(&delta_y);

            move |event: &Event| {
                let mouse_event = event.clone().dyn_into::<MouseEvent>().unwrap();
                delta_x.fetch_add(mouse_event.movement_x(), SeqCst);
                delta_y.fetch_add(mouse_event.movement_y(), SeqCst);
            }
        })
        .forget();

        Self { delta_x, delta_y }
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
}

pub fn emit_mouse_events(
    mut mouse_motion_events: EventWriter<MouseMotion>,
    mouse: Res<BrowserMouse>,
) {
    let delta = mouse.get_delta_and_reset();
    if delta != Vec2::ZERO {
        mouse_motion_events.send(MouseMotion { delta });
    }
}

pub struct PatchMouseMotionPlugin;

impl Plugin for PatchMouseMotionPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(emit_mouse_events.system())
            .insert_resource(BrowserMouse::new());
    }
}
