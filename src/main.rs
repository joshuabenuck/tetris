use glutin_window::GlutinWindow;
use graphics::character::CharacterCache;
use graphics::math::Matrix2d;
use graphics::text::Text;
use graphics::types::Color;
use graphics::{Context, DrawState, Graphics, Image, Transformed};
use opengl_graphics::{Filter, GlGraphics, GlyphCache, OpenGL, TextureSettings};
use piston::event_loop::{EventLoop, EventSettings, Events};
use piston::input::RenderEvent;
use piston::input::{Button, GenericEvent};
use piston::window::WindowSettings;
use rand::{rngs::ThreadRng, thread_rng, Rng};
use std::collections::HashSet;

pub struct Tetris {}

impl Tetris {
    fn new() -> Tetris {
        return Tetris {};
    }
}

const FPS: u32 = 25;
const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;
const BOXSIZE: u32 = 20;
const BOARDWIDTH: u32 = 10;
const BOARDHEIGHT: u32 = 20;
const BLANK: u8 = b'.';

#[cfg_attr(rustfmt, rustfmt_skip)]
const S_SHAPE: [[&'static [u8; 5]; 5]; 2] = [
    [b".....",
     b".....",
     b"..OO.",
     b".OO..",
     b"....."],
    [b".....",
     b"..O..",
     b"..OO.",
     b"...O.",
     b"....."],
];
#[cfg_attr(rustfmt, rustfmt_skip)]
const Z_SHAPE: [[&'static [u8; 5]; 5]; 2] = [
    [b".....",
     b".....",
     b".OO..",
     b"..OO.",
     b"....."],
    [b".....",
     b"..O..",
     b".OO..",
     b".O...",
     b"....."],
];
#[cfg_attr(rustfmt, rustfmt_skip)]
const I_SHAPE: [[&'static [u8; 5]; 5]; 2] = [
    [b"..O..",
     b"..O..",
     b"..O..",
     b"..O..",
     b"....."],
    [b".....",
     b".....",
     b"OOOO.",
     b".....",
     b"....."],
];

const MOVESIDEWAYSFREQ: f64 = 0.15;
const MOVEDOWNFREQ: f64 = 0.1;

const XMARGIN: u32 = (WIDTH - BOARDWIDTH * BOXSIZE) / 2;
const TOPMARGIN: u32 = HEIGHT - (BOARDHEIGHT - BOXSIZE) - 5;

const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const GRAY: [f32; 4] = [0.75, 0.75, 0.75, 1.0];
const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const LIGHTRED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const GREEN: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const LIGHTGREEN: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const BLUE: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const LIGHTBLUE: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const YELLOW: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const LIGHTYELLOW: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

const BORDERCOLOR: [f32; 4] = BLUE;
const BGCOLOR: [f32; 4] = BLACK;
const TEXTCOLOR: [f32; 4] = WHITE;
const TEXTSHADOWCOLOR: [f32; 4] = GRAY;
const COLORS: [[f32; 4]; 4] = [BLUE, GREEN, RED, YELLOW];
const LIGHTCOLORS: [[f32; 4]; 4] = [LIGHTBLUE, LIGHTGREEN, LIGHTRED, LIGHTYELLOW];

enum Shapes {
    S,
    Z,
    J,
    L,
    I,
    O,
    T,
}

fn main() {
    assert!(
        COLORS.len() == LIGHTCOLORS.len(),
        "Each color must have a light color!"
    );
    let mut settings = EventSettings::new();
    settings.set_lazy(true);
    settings.swap_buffers(true);
    settings.max_fps(1);
    settings.ups(1);
    let mut events = Events::new(settings);
    let opengl = OpenGL::V3_2;
    let settings = WindowSettings::new("Tetris", [WIDTH, HEIGHT])
        .exit_on_esc(true)
        .graphics_api(opengl);
    let mut window: GlutinWindow = settings.build().expect("Could not create window");
    let mut gl = GlGraphics::new(opengl);

    // let mut tetris = Tetris::new();

    let texture_settings = TextureSettings::new().filter(Filter::Nearest);
    let ref mut glyphs =
        GlyphCache::new("freesansbold.ttf", (), texture_settings).expect("Could not load font");
    let show_text = true;
    while let Some(e) = events.next(&mut window) {
        if show_text {
            if show_text_screen("Tetris", e, &mut gl, glyphs) {
                break;
            }
        }
    }
}

fn show_text_screen<E: GenericEvent>(
    text: &str,
    e: E,
    gl: &mut GlGraphics,
    glyphs: &mut GlyphCache,
) -> bool {
    if let Some(Button::Keyboard(_)) = e.press_args() {
        return true;
    }
    if let Some(args) = e.render_args() {
        let mut font = Text::new(1);
        gl.draw(args.viewport(), |c, g| {
            use graphics::clear;
            clear(BLACK, g);
            // let window_size = args.window_size;
            font.font_size = 100;
            font.color = TEXTSHADOWCOLOR;
            font.draw_center(
                text,
                glyphs,
                &c.draw_state,
                c.transform.trans((WIDTH / 2) as f64, (HEIGHT / 2) as f64),
                g,
            )
            .expect("Unable to draw string");
            font.color = TEXTCOLOR;
            font.draw_center(
                text,
                glyphs,
                &c.draw_state,
                c.transform
                    .trans((WIDTH / 2 - 3) as f64, (HEIGHT / 2 - 3) as f64),
                g,
            )
            .expect("Unable to draw string");
            font.font_size = 18;
            font.color = TEXTCOLOR;
            font.draw_center(
                "Press a key to play",
                glyphs,
                &c.draw_state,
                c.transform
                    .trans((WIDTH / 2) as f64, (HEIGHT / 2 + 100) as f64),
                g,
            )
            .expect("Unable to draw string");
            // gameboard_view.settings.position[0] =
            //     (window_size[0] - gameboard_view.settings.size) / 2.0;
            // gameboard_view.settings.position[1] =
            //     (window_size[1] - gameboard_view.settings.size) / 2.0;
            // gameboard_view.draw(&gameboard_controller, glyphs, &c, g);
        });
    }
    false
}

// Hack to workaround API limitations...
trait CenterText {
    fn draw_center<C, G>(
        &self,
        text: &str,
        cache: &mut C,
        draw_state: &DrawState,
        tranform: Matrix2d,
        g: &mut G,
    ) -> Result<(), C::Error>
    where
        C: CharacterCache,
        G: Graphics<Texture = <C as CharacterCache>::Texture>;
}

impl CenterText for Text {
    fn draw_center<C, G>(
        &self,
        text: &str,
        cache: &mut C,
        draw_state: &DrawState,
        transform: Matrix2d,
        g: &mut G,
    ) -> Result<(), C::Error>
    where
        C: CharacterCache,
        G: Graphics<Texture = <C as CharacterCache>::Texture>,
    {
        let mut width = 0.0;
        let mut space_size = 0.0;
        for ch in text.chars() {
            let character = cache.character(self.font_size, ch)?;
            width += character.advance_width();
            space_size = character.advance_width() - character.atlas_size[0];
        }
        width -= space_size;
        let transform = transform.trans(-width / 2.0, 0.0);
        self.draw(text, cache, draw_state, transform, g)
    }
}
