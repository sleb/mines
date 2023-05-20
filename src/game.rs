use std::ops::{Index, IndexMut};

use cursive::{
    align::HAlign,
    direction::Direction,
    event::{Event, EventResult, MouseButton, MouseEvent},
    views::{Button, Dialog, LinearLayout, PaddedView, Panel, SelectView},
    Cursive, Vec2, View, XY,
};
use rand::Rng;

#[derive(Debug)]
enum Difficulty {
    Beginner,
    Intermediate,
    Expert,
}

pub fn start_menu(s: &mut Cursive) {
    s.pop_layer();

    let select = SelectView::new()
        .h_align(HAlign::Left)
        .item("Beginner", Difficulty::Beginner)
        .item("Intermediate", Difficulty::Intermediate)
        .item("Expert", Difficulty::Expert)
        .on_submit(new_game);

    s.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(PaddedView::lrtb(
                    0,
                    0,
                    1,
                    0,
                    Panel::new(select).title("New Game"),
                ))
                .child(Button::new("Top Scores", top_scores))
                .child(Button::new("Quit", |s| s.quit())),
        )
        .title("Mines!"),
    );
}

fn top_scores(s: &mut Cursive) {
    s.pop_layer();

    s.add_layer(
        Dialog::text("Scores...")
            .title("High Scores")
            .button("Back", start_menu),
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CellContents {
    Bomb,
    Hint(u32),
}

impl Default for CellContents {
    fn default() -> Self {
        Self::Hint(0)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
enum CellState {
    #[default]
    Hidden,
    Revealed,
    Flagged,
}

#[derive(Debug, Default, Clone)]
struct Cell {
    contents: CellContents,
    state: CellState,
}

fn blow_up(s: &mut Cursive) {
    s.add_layer(Dialog::text("!!!! BOOOM !!!!").button("Try Again", start_menu));
}

struct Grid {
    size: (usize, usize),
    cells: Vec<Cell>,
}

const NUMBERS: [&str; 9] = [
    "[0]", "[1]", "[2]", "[3]", "[4]", "[5]", "[6]", "[7]", "[8]",
];

impl View for Grid {
    fn take_focus(&mut self, _: Direction) -> Result<EventResult, cursive::view::CannotFocus> {
        Ok(EventResult::Consumed(None))
    }

    fn on_event(&mut self, e: Event) -> EventResult {
        if let Event::Mouse {
            offset,
            position,
            event,
        } = e
        {
            if let Some(XY { x, y }) = position.checked_sub(offset) {
                let (r, c) = (y, x / 3);
                if r < self.size.0 && c < self.size.1 {
                    let cell = &mut self[(r, c)];
                    if cell.state != CellState::Revealed {
                        match event {
                            MouseEvent::Press(MouseButton::Left) => {
                                if cell.contents == CellContents::Bomb {
                                    return EventResult::with_cb(|s| blow_up(s));
                                }
                                self.reveal((r, c));
                            }
                            MouseEvent::Press(MouseButton::Right) => {
                                if cell.state == CellState::Flagged {
                                    cell.state = CellState::Hidden;
                                } else {
                                    cell.state = CellState::Flagged;
                                }

                                return EventResult::Consumed(None);
                            }
                            _ => (),
                        };
                    }
                }
            }
        }

        EventResult::Ignored
    }

    fn draw(&self, printer: &cursive::Printer) {
        let (r, c) = self.size;
        for x in 0..r {
            for y in 0..c {
                let cell = &self[(x, y)];
                let text = match cell.state {
                    CellState::Flagged => "[~]",
                    CellState::Hidden => "[#]",
                    CellState::Revealed => match cell.contents {
                        CellContents::Hint(n) => NUMBERS[n as usize],
                        CellContents::Bomb => "[*]",
                    },
                };
                printer.print((y * 3, x), text);
            }
        }
    }

    fn required_size(&mut self, _: Vec2) -> Vec2 {
        let (r, c) = self.size;
        Vec2::new(c * 3, r)
    }

    fn layout(&mut self, _: Vec2) {}

    fn needs_relayout(&self) -> bool {
        true
    }

    fn call_on_any(&mut self, _: &cursive::view::Selector, _: cursive::event::AnyCb) {}

    fn focus_view(
        &mut self,
        _: &cursive::view::Selector,
    ) -> Result<EventResult, cursive::view::ViewNotFound> {
        Err(cursive::view::ViewNotFound)
    }

    fn important_area(&self, view_size: Vec2) -> cursive::Rect {
        cursive::Rect::from_size((0, 0), view_size)
    }

    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

impl Grid {
    fn new(size: (usize, usize)) -> Grid {
        let (r, c) = size;
        let cells = vec![Cell::default(); r * c];

        Grid { size, cells }
    }

    fn reveal(&mut self, index: (usize, usize)) {
        let cell = &mut self[index];
        cell.state = CellState::Revealed;

        if let CellContents::Hint(0) = cell.contents {
            let mut stack = self.neighbors(index);
            while let Some(current) = stack.pop() {
                let current_cell = &mut self[current];
                if let CellContents::Hint(0) = current_cell.contents {
                    current_cell.state = CellState::Revealed;
                    stack.append(&mut self.neighbors(current));
                }
            }
        }
    }

    fn neighbors(&self, index: (usize, usize)) -> Vec<(usize, usize)> {
        let (r, c) = index;
        let mut res = Vec::with_capacity(4);
        if r > 0 {
            res.push((r - 1, c));

            if c > 0 {
                res.push((r - 1, c - 1));
            }

            if r < self.size.0 - 1 {
                res.push((r - 1, c + 1));
            }
        }

        if c > 0 {
            res.push((r, c - 1));
        }

        if c < self.size.1 - 1 {
            res.push((r, c + 1));
        }

        if r < self.size.0 - 1 {
            res.push((r + 1, c));

            if c > 0 {
                res.push((r + 1, c - 1));
            }

            if c < self.size.1 + 1 {
                res.push((r + 1, c + 1))
            }
        }

        res
    }
}

impl Index<(usize, usize)> for Grid {
    type Output = Cell;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.cells[index.0 * self.size.0 + index.1]
    }
}

impl IndexMut<(usize, usize)> for Grid {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.cells[index.0 * self.size.0 + index.1]
    }
}

struct Config {
    size: (usize, usize),
    num_bombs: u32,
}

impl From<&Difficulty> for Config {
    fn from(value: &Difficulty) -> Self {
        match value {
            Difficulty::Beginner => Config {
                size: (9, 9),
                num_bombs: 10,
            },
            Difficulty::Intermediate => Config {
                size: (16, 16),
                num_bombs: 40,
            },
            Difficulty::Expert => Config {
                size: (16, 30),
                num_bombs: 99,
            },
        }
    }
}

fn place_bombs_rnd<R: Rng>(rng: R, grid: &mut Grid, num_bombs: u32) {
    let mut rng = rng;

    let (r, c) = grid.size;
    let mut bombs_placed = 0;
    while bombs_placed < num_bombs {
        let index = (rng.gen_range(0..r), rng.gen_range(0..c));
        let cell = &mut grid[index];
        if cell.contents != CellContents::Bomb {
            cell.contents = CellContents::Bomb;

            for neighbor in grid.neighbors(index) {
                let neighbor_cell = &mut grid[neighbor];
                if let CellContents::Hint(n) = neighbor_cell.contents {
                    neighbor_cell.contents = CellContents::Hint(n + 1);
                }
            }
            bombs_placed += 1;
        }
    }
}

fn new_game(s: &mut Cursive, d: &Difficulty) {
    s.pop_layer();

    let config = Config::from(d);
    let mut grid = Grid::new(config.size);

    let rng = rand::thread_rng();
    place_bombs_rnd(rng, &mut grid, config.num_bombs);

    s.add_layer(Dialog::around(
        LinearLayout::vertical()
            .child(Panel::new(grid))
            .child(Button::new("Quit", |s| s.quit())),
    ));
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_neighbors() {
        let size = (5, 5);
        let grid = Grid::new(size);
    }
}
