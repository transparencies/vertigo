use vertigo::{computed::Computed, VDomElement, Css};
use vertigo_html::html_component;

use self::config::Config;
use super::state::{Cell, Sudoku, sudoku_square::SudokuSquare, tree_box::TreeBoxIndex};

pub mod config;
pub mod render_cell_value;
pub mod render_cell_possible;

fn css_center() -> Css {
    Css::one("
        display: flex;
        justify-content: center;
    ")
}

fn css_wrapper() -> Css {
    let config = Config::new();
    Css::new(format!("
        display: grid;
        grid-template-columns: 1fr 1fr 1fr;
        flex-shrink: 0;

        width: {}px;
        height: {}px;

        border: 2px solid blue;
    ", config.all_width, config.all_width))
}

fn css_item_wrapper() -> Css {
    let config = Config::new();
    Css::new(format!("
        border: {}px solid black;

        width: {}px;
        height: {}px;

        display: grid;
        grid-template-columns: 1fr 1fr 1fr;
        flex-shrink: 0;
    ", config.group_border_size, config.group_width_size, config.group_width_size))
}

fn css_cell_wrapper() -> Css {
    let config = Config::new();
    Css::new(format!("
        border: {}px solid green;
        width: {}px;
        height: {}px;
    ", config.item_border_size, config.item_width_size, config.item_width_size))
}

fn render_cell(item: &Computed<Cell>) -> VDomElement {
    let value = *item.get_value().number.value.get_value();

    // log::warn!("cell {:?}", value);
    if let Some(value) = value {
        return render_cell_value::render_cell_value(value, item);
    }

    render_cell_possible::render_cell_possible(item)
}

fn render_group(group: &Computed<SudokuSquare<Cell>>) -> VDomElement {
    //log::info!("render group");

    let get_cell = |group: &Computed<SudokuSquare<Cell>>, x: TreeBoxIndex, y: TreeBoxIndex| -> Computed<Cell> {
        group.clone().map(move |state| {
            state.get_value().get_from(x, y)
        })
    };

    html_component! {
        <div css={css_item_wrapper()}>
            <div css={css_cell_wrapper()}>
                <component {render_cell} data={get_cell(group, TreeBoxIndex::First,  TreeBoxIndex::First)} />
            </div>
            <div css={css_cell_wrapper()}>
                <component {render_cell} data={get_cell(group, TreeBoxIndex::First,  TreeBoxIndex::Middle)} />
            </div>
            <div css={css_cell_wrapper()}>
                <component {render_cell} data={get_cell(group, TreeBoxIndex::First,  TreeBoxIndex::Last)} />
            </div>
            <div css={css_cell_wrapper()}>
                <component {render_cell} data={get_cell(group, TreeBoxIndex::Middle,  TreeBoxIndex::First)} />
            </div>
            <div css={css_cell_wrapper()}>
                <component {render_cell} data={get_cell(group, TreeBoxIndex::Middle,  TreeBoxIndex::Middle)} />
            </div>
            <div css={css_cell_wrapper()}>
                <component {render_cell} data={get_cell(group, TreeBoxIndex::Middle,  TreeBoxIndex::Last)} />
            </div>
            <div css={css_cell_wrapper()}>
                <component {render_cell} data={get_cell(group, TreeBoxIndex::Last,  TreeBoxIndex::First)} />
            </div>
            <div css={css_cell_wrapper()}>
                <component {render_cell} data={get_cell(group, TreeBoxIndex::Last,  TreeBoxIndex::Middle)} />
            </div>
            <div css={css_cell_wrapper()}>
                <component {render_cell} data={get_cell(group, TreeBoxIndex::Last,  TreeBoxIndex::Last)} />
            </div>
        </div>
    }
}

pub fn main_render(sudoku: &Computed<Sudoku>) -> VDomElement {
    let get_group = |sudoku: &Computed<Sudoku>, x: TreeBoxIndex, y: TreeBoxIndex| -> Computed<SudokuSquare<Cell>> {
        sudoku.clone().map(move |state| {
            state.get_value().grid.get_from(x, y)
        })
    };

    html_component! {
        <div css={css_center()}>
            <div css={css_wrapper()}>
                <component {render_group} data={get_group(sudoku, TreeBoxIndex::First,  TreeBoxIndex::First)} />
                <component {render_group} data={get_group(sudoku, TreeBoxIndex::First,  TreeBoxIndex::Middle)} />
                <component {render_group} data={get_group(sudoku, TreeBoxIndex::First,  TreeBoxIndex::Last)} />
                <component {render_group} data={get_group(sudoku, TreeBoxIndex::Middle,  TreeBoxIndex::First)} />
                <component {render_group} data={get_group(sudoku, TreeBoxIndex::Middle,  TreeBoxIndex::Middle)} />
                <component {render_group} data={get_group(sudoku, TreeBoxIndex::Middle,  TreeBoxIndex::Last)} />
                <component {render_group} data={get_group(sudoku, TreeBoxIndex::Last,  TreeBoxIndex::First)} />
                <component {render_group} data={get_group(sudoku, TreeBoxIndex::Last,  TreeBoxIndex::Middle)} />
                <component {render_group} data={get_group(sudoku, TreeBoxIndex::Last,  TreeBoxIndex::Last)} />
            </div>
        </div>
    }
}

fn css_sudoku_example() -> Css {
    Css::one("
        border: 1px solid black;
        padding: 10px;
        margin: 10px 0;
    ")
}

fn css_sudoku_example_button() -> Css {
    Css::one("
        margin: 5px;
        cursor: pointer;
    ")
}
pub fn examples_render(sudoku: &Computed<Sudoku>) -> VDomElement {
    let sudoku = sudoku.get_value();

    let clear = {
        let sudoku = sudoku.clone();
        move || {
            sudoku.clear();
        }
    };

    let example1 = {
        let sudoku = sudoku.clone();
        move || { sudoku.example1(); }
    };

    let example2 = {
        let sudoku = sudoku.clone();
        move || { sudoku.example2(); }
    };

    let example3 = {
        move || { sudoku.example3(); }
    };

    html_component! {
        <div css={css_sudoku_example()}>
            <button css={css_sudoku_example_button()} onClick={clear}> Clear </button>
            <button css={css_sudoku_example_button()} onClick={example1}> Example 1 </button>
            <button css={css_sudoku_example_button()} onClick={example2}> Example 2 </button>
            <button css={css_sudoku_example_button()} onClick={example3}> Example 3 </button>
        </div>
    }
}