# Bevy MouseMove patch

Unfortunately, [Bevy](https://github.com/bevyengine/bevy) as of 0.5.0 does not
write
[`bevy::input::mouse::MouseMotion`](https://docs.rs/bevy/0.5.0/bevy/input/mouse/struct.MouseMotion.html)
events in browsers with a WASM/WebGL target
([GitHub issue](https://github.com/bevyengine/bevy/issues/1166)).

If you've stumbled across this through Google (like me), you can use this crate
to add in the `MouseMotion` events:

```rust
let mut app = App::build();

#[cfg(target_arch = "wasm32")]
app.add_plugin(bevy_webgl2::WebGL2Plugin)
   .add_plugin(PatchMouseMotionPlugin);
```

## Solution and Acknowledgement

I created a plugin from samcarey's example solution posted on
[the bevy GitHub issue](https://github.com/bevyengine/bevy/issues/1166).

It simply uses `web_sys` and `gloo` to attach a mouse move listener to the
document. The underlying `winit` implementation is supposed to attach listeners
directly to the canvas, but this was easier to patch in.

## Why is this happening??

I don't know why this is happening! I spent a few hours trying to figure it out,
but I couldn't figure it out. It seems `winit`
[is properly emitting the `DeviceEvent`s](https://github.com/rust-windowing/winit/issues/2036),
so the problem is likely in bevy. Or maybe it's been fixed on main, but I
couldn't get my project to use a local copy of bevy on the main branch without a
substantial amount of work. Either way, the currently published version of bevy
doesn't seem to do it (on my machine), so this patch is helpful to continue
making whatever game you're trying to make.
