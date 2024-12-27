use web_sys::HtmlTextAreaElement;
use yew::prelude::*;
use yew_hooks::prelude::*;

fn main() {
    yew::Renderer::<App>::new().render();
}

#[function_component]
fn App() -> Html {
    let ws = use_websocket("ws://127.0.0.1:7878".to_string());
    let msg = use_state(|| "".to_string());
    let messages = use_list(vec![]);

    {
        let ws = ws.clone();
        let messages = messages.clone();
        use_effect_with(ws.message, move |msg| {
            if let Some(msg) = &**msg {
                messages.push(msg.clone());
            }
        });
    }

    let onchange = {
        let msg = msg.clone();
        Callback::from(move |e: Event| {
            let textarea = e.target_dyn_into::<HtmlTextAreaElement>().unwrap();
            msg.set(textarea.value());
        })
    };

    let onclick = {
        let ws = ws.clone();
        let msg = msg.clone();
        Callback::from(move |_| {
            ws.send((*msg).clone());
            msg.set("".to_string());
        })
    };

    html! {
        <>
            {
                for messages.current().iter().map(|msg| {
                    html! { <p>{ msg }</p> }
                })
            }
            <textarea { onchange } value={ (*msg).clone() }></textarea>
            <button { onclick } disabled={ *ws.ready_state != UseWebSocketReadyState::Open }>{ "Send" }</button>
        </>
    }
}
