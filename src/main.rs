use glutin_window::GlutinWindow;
use graphics::character::CharacterCache;
use graphics::math::Matrix2d;
use graphics::text::Text;
use graphics::types::Color;
use graphics::{Context, DrawState, Graphics, Transformed};
use opengl_graphics::{Filter, GlGraphics, GlyphCache, OpenGL, TextureSettings};
use piston::event_loop::{EventLoop, EventSettings, Events};
use piston::input::{keyboard::Key, Button, GenericEvent};
use piston::window::WindowSettings;
use rand::{rngs::ThreadRng, Rng};
use std::clone::Clone;
use std::time::{Duration, Instant};

const FPS: u64 = 25;
const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;
const BOXSIZE: u32 = 20;
const BOARDWIDTH: usize = 10;
const BOARDHEIGHT: usize = 20;
const BLANK: u8 = b'.';
const TEMPLATEWIDTH: usize = 5;
const TEMPLATEHEIGHT: usize = 5;

#[cfg_attr(rustfmt, rustfmt_skip)]
const S_SHAPE: [[&'static [u8; TEMPLATEWIDTH]; TEMPLATEHEIGHT]; 2] = [
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
const Z_SHAPE: [[&'static [u8; TEMPLATEWIDTH]; TEMPLATEHEIGHT]; 2] = [
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
const I_SHAPE: [[&'static [u8; TEMPLATEWIDTH]; TEMPLATEHEIGHT]; 2] = [
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
#[cfg_attr(rustfmt, rustfmt_skip)]
const O_SHAPE: [[&'static [u8; TEMPLATEWIDTH]; TEMPLATEHEIGHT]; 1] = [
    [b".....",
     b".....",
     b".OO..",
     b".OO..",
     b"....."],
];
#[cfg_attr(rustfmt, rustfmt_skip)]
const J_SHAPE: [[&'static [u8; TEMPLATEWIDTH]; TEMPLATEHEIGHT]; 4] = [
    [b".....",
     b".O...",
     b".OOO.",
     b".....",
     b"....."],
    [b".....",
     b".OO..",
     b".O...",
     b".O...",
     b"....."],
    [b".....",
     b".....",
     b".OOO.",
     b"...O.",
     b"....."],
    [b".....",
     b"..O..",
     b"..O..",
     b".OO..",
     b"....."],
];
#[cfg_attr(rustfmt, rustfmt_skip)]
const L_SHAPE: [[&'static [u8; TEMPLATEWIDTH]; TEMPLATEHEIGHT]; 4] = [
    [b".....",
     b"...O.",
     b".OOO.",
     b".....",
     b"....."],
    [b".....",
     b"..O..",
     b"..O..",
     b"..OO.",
     b"....."],
    [b".....",
     b".....",
     b".OOO.",
     b".O...",
     b"....."],
    [b".....",
     b".OO..",
     b"..O..",
     b"..O..",
     b"....."],
];
#[cfg_attr(rustfmt, rustfmt_skip)]
const T_SHAPE: [[&'static [u8; TEMPLATEWIDTH]; TEMPLATEHEIGHT]; 4] = [
    [b".....",
     b"..O..",
     b".OOO.",
     b".....",
     b"....."],
    [b".....",
     b"..O..",
     b"..OO.",
     b"..O..",
     b"....."],
    [b".....",
     b".....",
     b".OOO.",
     b"..O..",
     b"....."],
    [b".....",
     b"..O..",
     b".OO..",
     b"..O..",
     b"....."],
];

const MOVESIDEWAYSFREQ: Duration = Duration::from_millis(150);
const MOVEDOWNFREQ: Duration = Duration::from_millis(100);

const XMARGIN: u32 = (WIDTH - BOARDWIDTH as u32 * BOXSIZE) / 2;
const TOPMARGIN: u32 = HEIGHT - (BOARDHEIGHT as u32 * BOXSIZE) - 5;

const WHITE: Color = [1.0, 1.0, 1.0, 1.0];
const GRAY: Color = [0.72, 0.72, 0.72, 1.0];
const BLACK: Color = [0.0, 0.0, 0.0, 1.0];
const RED: Color = [0.60, 0.60, 0.60, 1.0];
const LIGHTRED: Color = [0.68, 0.68, 0.68, 1.0];
const GREEN: Color = [0.0, 0.60, 0.0, 1.0];
const LIGHTGREEN: Color = [0.1, 0.68, 0.0, 1.0];
const BLUE: Color = [0.0, 0.0, 0.60, 1.0];
const LIGHTBLUE: Color = [0.1, 0.1, 0.68, 1.0];
const YELLOW: Color = [0.60, 0.68, 0.0, 1.0];
const LIGHTYELLOW: Color = [0.68, 0.68, 0.0, 1.0];

const BORDERCOLOR: Color = BLUE;
const BGCOLOR: Color = BLACK;
const TEXTCOLOR: Color = WHITE;
const TEXTSHADOWCOLOR: Color = GRAY;
const COLORS: [Color; 4] = [BLUE, GREEN, RED, YELLOW];
const LIGHTCOLORS: [Color; 4] = [LIGHTBLUE, LIGHTGREEN, LIGHTRED, LIGHTYELLOW];

#[derive(Clone)]
enum Shape {
    S,
    Z,
    J,
    L,
    I,
    O,
    T,
}

impl Shape {
    fn template(&self) -> Vec<[&'static [u8; TEMPLATEWIDTH]; TEMPLATEHEIGHT]> {
        match self {
            Shape::S => S_SHAPE.to_vec(),
            Shape::Z => Z_SHAPE.to_vec(),
            Shape::J => J_SHAPE.to_vec(),
            Shape::L => L_SHAPE.to_vec(),
            Shape::I => I_SHAPE.to_vec(),
            Shape::O => O_SHAPE.to_vec(),
            Shape::T => T_SHAPE.to_vec(),
        }
    }
}

#[derive(PartialEq)]
enum State {
    TitleScreen,
    Run,
    Paused,
    GameOver,
    Quit,
}

fn main() {
    assert!(
        COLORS.len() == LIGHTCOLORS.len(),
        "Each color must have a light color!"
    );
    let mut settings = EventSettings::new();
    settings.set_lazy(true);
    settings.swap_buffers(true);
    settings.max_fps(FPS);
    settings.ups(FPS);
    let mut events = Events::new(settings);
    let opengl = OpenGL::V3_2;
    let settings = WindowSettings::new("Tetris", [WIDTH, HEIGHT])
        .exit_on_esc(true)
        .graphics_api(opengl);
    let mut window: GlutinWindow = settings.build().expect("Could not create window");
    let mut gl = GlGraphics::new(opengl);

    let mut tetris = Tetris::new();

    let texture_settings = TextureSettings::new().filter(Filter::Nearest);
    let ref mut glyphs =
        GlyphCache::new("freesansbold.ttf", (), texture_settings).expect("Could not load font");
    let mut state = State::TitleScreen;
    while let Some(e) = events.next(&mut window) {
        use State::*;
        state = match state {
            TitleScreen => {
                show_text_screen("Tetris", e, &mut gl, glyphs, Run).unwrap_or(TitleScreen)
            }
            Run => tetris.run(e, &mut gl, glyphs),
            Paused => show_text_screen("Paused", e, &mut gl, glyphs, Run).unwrap_or(Paused),
            GameOver => {
                let next_state =
                    show_text_screen("Game Over", e, &mut gl, glyphs, Run).unwrap_or(GameOver);
                if next_state == Run {
                    tetris = Tetris::new();
                }
                next_state
            }
            Quit => break,
        }
    }
}

struct Piece {
    shape: Shape,
    rotation: usize,
    x: usize,
    y: isize,
    color: usize,
}

impl DrawBox for Piece {}

impl Piece {
    fn new(rng: &mut ThreadRng) -> Piece {
        use Shape::*;
        let shapes = [I, J, S, Z, O, T, L];
        let shape = &shapes[rng.gen_range(0, shapes.len())];
        Piece {
            shape: shape.clone(),
            rotation: rng.gen_range(0, shape.template().len()),
            x: BOARDWIDTH / 2 - TEMPLATEWIDTH / 2,
            y: -2,
            color: rng.gen_range(0, COLORS.len()),
        }
    }

    fn draw(&self, c: &Context, g: &mut GlGraphics) {
        let (px, py) = xy_to_pxy(self.x, self.y as usize);
        self.draw_at(px, py, c, g);
    }

    fn draw_at(&self, px: u32, py: u32, c: &Context, g: &mut GlGraphics) {
        let to_draw = self.shape.template()[self.rotation];
        for x in 0..TEMPLATEWIDTH {
            for y in 0..TEMPLATEHEIGHT {
                if to_draw[y][x] != BLANK {
                    self.draw_box(
                        px + (x as u32 * BOXSIZE),
                        py + (y as u32 * BOXSIZE),
                        Some(self.color),
                        &c,
                        g,
                    );
                }
            }
        }
    }
}

fn xy_to_pxy(x: usize, y: usize) -> (u32, u32) {
    (
        XMARGIN + (x as u32 * BOXSIZE),
        TOPMARGIN + (y as u32 * BOXSIZE),
    )
}

trait DrawBox {
    fn draw_box(&self, px: u32, py: u32, color: Option<usize>, c: &Context, g: &mut GlGraphics) {
        use graphics::Rectangle;
        match color {
            None => {}
            Some(color) => {
                let border_rect = [
                    px as f64 + 1.0f64,
                    py as f64 + 1.0f64,
                    BOXSIZE as f64 - 1.0f64,
                    BOXSIZE as f64 - 1.0f64,
                ];
                let box_rect = [
                    px as f64 + 1.0f64,
                    py as f64 + 1.0f64,
                    BOXSIZE as f64 - 4.0f64,
                    BOXSIZE as f64 - 4.0f64,
                ];

                Rectangle::new(COLORS[color]).draw(border_rect, &c.draw_state, c.transform, g);
                Rectangle::new(LIGHTCOLORS[color]).draw(box_rect, &c.draw_state, c.transform, g);
            }
        }
    }
}

struct Board([[Option<usize>; BOARDHEIGHT]; BOARDWIDTH]);

impl DrawBox for Board {}

impl Board {
    fn new() -> Board {
        Board([[None; BOARDHEIGHT]; BOARDWIDTH])
    }

    fn contains(&self, x: isize, y: isize) -> bool {
        // How to encode this constraint into a type?
        x >= 0 && x < BOARDWIDTH as isize && y < BOARDHEIGHT as isize
    }

    fn is_valid_position(&self, piece: &Piece, adj_x: isize, adj_y: isize) -> bool {
        for x in 0..TEMPLATEWIDTH {
            for y in 0..TEMPLATEHEIGHT {
                let is_above_board = (y as isize + piece.y + adj_y) < 0;
                if is_above_board || piece.shape.template()[piece.rotation][y][x] == BLANK {
                    continue;
                }
                let new_x = x as isize + piece.x as isize + adj_x;
                let new_y = y as isize + piece.y + adj_y;
                if !self.contains(new_x, new_y) {
                    return false;
                }
                if !self.0[new_x as usize][new_y as usize].is_none() {
                    return false;
                }
            }
        }
        true
    }

    fn add(&mut self, piece: &Piece) {
        for x in 0..TEMPLATEWIDTH {
            for y in 0..TEMPLATEHEIGHT {
                if piece.shape.template()[piece.rotation][y][x] != BLANK {
                    // Not sure why the Python version doesn't need this guard clause.
                    if self.contains(x as isize + piece.x as isize, y as isize + piece.y) {
                        self.0[x + piece.x][y + piece.y as usize] = Some(piece.color);
                    }
                }
            }
        }
    }

    fn remove_complete_lines(&mut self) -> u32 {
        let mut count_removed = 0;
        let mut y = BOARDHEIGHT - 1;
        loop {
            if self.is_complete_line(y) {
                for pulldown_y in (1..y + 1).rev() {
                    for x in 0..BOARDWIDTH {
                        self.0[x][pulldown_y] = self.0[x][pulldown_y - 1]
                    }
                }
                for x in 0..BOARDWIDTH {
                    self.0[x][0] = None
                }
                count_removed += 1;
            } else {
                if y == 0 {
                    break;
                }
                y -= 1;
            }
        }
        count_removed
    }

    fn is_complete_line(&self, y: usize) -> bool {
        for x in 0..BOARDWIDTH {
            if self.0[x][y].is_none() {
                return false;
            }
        }
        true
    }

    fn draw(&self, c: &Context, g: &mut GlGraphics) {
        use graphics::Rectangle;

        let border_rect = [
            (XMARGIN - 3) as f64,
            (TOPMARGIN - 7) as f64,
            ((BOARDWIDTH as u32 * BOXSIZE) + 8) as f64,
            ((BOARDHEIGHT as u32 * BOXSIZE) + 8) as f64,
        ];

        Rectangle::new_border(BORDERCOLOR, 5.0).draw(border_rect, &c.draw_state, c.transform, g);

        let board_rect = [
            XMARGIN as f64,
            TOPMARGIN as f64,
            (BOARDWIDTH as u32 * BOXSIZE) as f64,
            (BOARDHEIGHT as u32 * BOXSIZE) as f64,
        ];

        Rectangle::new(BGCOLOR).draw(board_rect, &c.draw_state, c.transform, g);
        for x in 0..BOARDWIDTH {
            for y in 0..BOARDHEIGHT {
                let (pixel_x, pixel_y) = xy_to_pxy(x, y);
                self.draw_box(pixel_x, pixel_y, self.0[x][y], c, g);
            }
        }
    }
}

pub struct Tetris {
    board: Board,
    rng: ThreadRng,
    last_fall_time: Instant,
    last_move_down_time: Instant,
    last_move_sideways_time: Instant,
    moving: Moving,
    score: u32,
    falling_piece: Option<Piece>,
    next_piece: Piece,
    level: u32,
    fall_freq: Duration,
}

impl Tetris {
    fn new() -> Tetris {
        let last_move_down_time = Instant::now();
        let last_move_sideways_time = Instant::now();
        let last_fall_time = Instant::now();
        let moving = Moving::Not;
        let score: u32 = 0;
        let (level, fall_freq) = calculate_level_and_fall_freq(score);
        let mut rng = rand::thread_rng();
        let falling_piece = Some(Piece::new(&mut rng));
        let next_piece = Piece::new(&mut rng);
        Tetris {
            board: Board::new(),
            rng,
            last_fall_time,
            last_move_down_time,
            last_move_sideways_time,
            moving,
            score,
            falling_piece,
            next_piece,
            level,
            fall_freq,
        }
    }

    fn run<E: GenericEvent>(
        &mut self,
        e: E,
        gl: &mut GlGraphics,
        glyphs: &mut GlyphCache,
    ) -> State {
        use std::mem::replace;
        if self.falling_piece.is_none() {
            self.falling_piece = Some(replace(&mut self.next_piece, Piece::new(&mut self.rng)));
            self.last_fall_time = Instant::now();

            if !self
                .board
                .is_valid_position(self.falling_piece.as_ref().unwrap(), 0, 0)
            {
                return State::GameOver;
            }
        }
        let fp = self.falling_piece.as_mut().unwrap();
        if let Some(Button::Keyboard(key)) = e.release_args() {
            match key {
                Key::P => return State::Paused,
                Key::Left | Key::Right | Key::Down => self.moving = Moving::Not,
                _ => {}
            }
        }
        if let Some(Button::Keyboard(key)) = e.press_args() {
            match key {
                Key::P => return State::Paused,
                Key::Left | Key::A => {
                    if self.board.is_valid_position(fp, -1, 0) {
                        fp.x -= 1;
                        self.moving = Moving::Left;
                        self.last_move_sideways_time = Instant::now();
                    }
                }
                Key::Right | Key::D => {
                    if self.board.is_valid_position(fp, 1, 0) {
                        fp.x += 1;
                        self.moving = Moving::Right;
                        self.last_move_sideways_time = Instant::now();
                    }
                }
                Key::Up | Key::W => {
                    fp.rotation = (fp.rotation + 1) % fp.shape.template().len();
                    if !self.board.is_valid_position(fp, 0, 0) {
                        fp.rotation = (fp.rotation - 1) % fp.shape.template().len();
                    }
                }
                Key::Q => {
                    fp.rotation = (fp.rotation - 1) % fp.shape.template().len();
                    if !self.board.is_valid_position(fp, 0, 0) {
                        fp.rotation = (fp.rotation + 1) % fp.shape.template().len();
                    }
                }
                Key::Down => {
                    self.moving = Moving::Down;
                    if self.board.is_valid_position(fp, 0, 1) {
                        fp.y += 1;
                    }
                    self.last_move_down_time = Instant::now();
                }
                Key::Space => {
                    self.moving = Moving::Not;
                    for i in 1..BOARDHEIGHT {
                        if !self.board.is_valid_position(fp, 0, i as isize) {
                            break;
                        }
                        fp.y += i as isize - 1;
                    }
                }
                _ => {}
            }
        }
        if (self.moving == Moving::Left || self.moving == Moving::Right)
            && ((Instant::now() - self.last_move_sideways_time) > MOVESIDEWAYSFREQ)
        {
            if self.moving == Moving::Left && self.board.is_valid_position(fp, -1, 0) {
                fp.x -= 1;
            }
            if self.moving == Moving::Right && self.board.is_valid_position(fp, 1, 0) {
                fp.x += 1;
            }
            self.last_move_sideways_time = Instant::now();
        }

        if self.moving == Moving::Down
            && ((Instant::now() - self.last_move_sideways_time) > MOVEDOWNFREQ)
            && self.board.is_valid_position(fp, 0, 1)
        {
            fp.y += 1;
            self.last_move_down_time = Instant::now();
        }
        drop(fp);

        if (Instant::now() - self.last_fall_time) > self.fall_freq {
            let fp = self.falling_piece.as_mut().unwrap();
            if !self.board.is_valid_position(fp, 0, 1) {
                self.board.add(fp);
                self.score += self.board.remove_complete_lines();
                let (level, fall_freq) = calculate_level_and_fall_freq(self.score);
                self.level = level;
                self.fall_freq = fall_freq;
                self.falling_piece = None;
                self.moving = Moving::Not;
            } else {
                fp.y += 1;
                self.last_fall_time = Instant::now();
            }
        }

        if let Some(args) = e.render_args() {
            let viewport = args.viewport();
            gl.draw(viewport, |c, g| {
                use graphics::clear;
                clear(BLACK, g);
                self.board.draw(&c, g);
                self.draw_status(&c, g, glyphs);
                let mut font = Text::new(18);
                font.color = TEXTCOLOR;
                font.draw(
                    "Next:",
                    glyphs,
                    &c.draw_state,
                    c.transform.trans((WIDTH - 120) as f64, 80.0f64),
                    g,
                )
                .expect("Unable to draw string");
                self.next_piece.draw_at(WIDTH - 120, 100, &c, g);
                if !self.falling_piece.is_none() {
                    self.falling_piece.as_ref().unwrap().draw(&c, g);
                }
            });
        }
        State::Run
    }

    fn draw_status(&self, c: &Context, g: &mut GlGraphics, glyphs: &mut GlyphCache) {
        let mut font = Text::new(18);
        font.color = TEXTCOLOR;
        font.draw_center(
            format!("Score: {}", self.score).as_str(),
            glyphs,
            &c.draw_state,
            c.transform.trans((WIDTH - 150) as f64, 20.0f64),
            g,
        )
        .expect("Unable to draw string");
        font.draw_center(
            format!("Level: {}", self.level).as_str(),
            glyphs,
            &c.draw_state,
            c.transform.trans((WIDTH - 150) as f64, 50.0f64),
            g,
        )
        .expect("Unable to draw string");
    }
}

fn calculate_level_and_fall_freq(score: u32) -> (u32, Duration) {
    let level = score / 10 + 1;
    let fall_freq = Duration::from_secs_f64(0.27 - (level as f64 * 0.02));
    (level, fall_freq)
}

#[derive(PartialEq)]
enum Moving {
    Down,
    Left,
    Right,
    Not,
}

fn show_text_screen<E: GenericEvent>(
    text: &str,
    e: E,
    gl: &mut GlGraphics,
    glyphs: &mut GlyphCache,
    next_state: State,
) -> Option<State> {
    if let Some(Button::Keyboard(_)) = e.press_args() {
        return Some(next_state);
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
    None
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
