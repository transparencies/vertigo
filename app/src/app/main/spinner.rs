use vertigo::Css;

use vertigo_html::{html_element, NodeAttr};

fn spinner_css() -> Css {
    Css::one("
        width: 40px;
        height: 40px;
        background-color: #d26913;

        border-radius: 100%;
        animation: 1.0s infinite ease-in-out {
            0% {
                -webkit-transform: scale(0);
                transform: scale(0);
            }
            100% {
                -webkit-transform: scale(1.0);
                transform: scale(1.0);
                opacity: 0;
            }
        };
    ")
}

pub fn spinner() -> NodeAttr {
    html_element! {
        <div css={spinner_css()}>
        </div>
    }
}