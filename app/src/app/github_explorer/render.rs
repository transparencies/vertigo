use vertigo::{
    computed::Computed,
    VDomElement,
    Css,
};
use vertigo_html::{Inline, html_component};

use super::state::{State, Resource};

fn wrapper() -> Css {
    Css::one("
        border: 1px solid black;
        margin: 20px 0;
        padding: 10px;
    ")
}

fn input_css() -> Css {
    Css::one("
        margin-left: 10px;
    ")
}

fn button_css() -> Css {
    Css::one("
        margin: 0 10px;
        cursor: pointer;
    ")
}

fn text_css() -> Css {
    Css::one("
        width: 600px;
        height: 300px;
        border: 1px solid black;
        padding: 5px;
        margin: 10px;
    ")
}

pub fn render(state: &Computed<State>) -> VDomElement {
    let state = state.get_value();

    let on_input_callback = {
        let value = state.repo_input.clone();
        move |new_value: String| {
            log::info!(" nowa wartosc3 {}", new_value);
            value.set_value(new_value);
        }
    };

    let on_show = {
        let value = state.repo_input.get_value();
        let state_inner = state.clone();
        move || {
            log::info!(" nowa wartosc3 {}", value);
            state_inner.repo_shown.set_value((*value).clone());
        }
    };

    let repo_input = state.repo_input.get_value();
    let repo_shown = state.repo_shown.get_value();

    let commit_sha = match repo_shown.as_str() {
        "" => "".to_string(),
        _ => match &*state.data.get_value(&repo_shown).get_value() {
            Resource::Loading => "Loading...".to_string(),
            Resource::Ready(branch) => branch.commit.sha.clone(),
            Resource::Failed(err) => format!("Error: {}", err),
        }
    };

    html_component! {
        <div css={wrapper()}>
            { "Enter author/repo tuple:" }
            <input css={input_css()} value={(*repo_input).as_str()} onInput={on_input_callback} />
            <button css={button_css()} onClick={on_show}>Fetch</button>
            <div css={button_css()}>
                { repo_shown.as_str() }
            </div>
            <div css={text_css()}>
                { commit_sha }
            </div>
        </div>
    }
}