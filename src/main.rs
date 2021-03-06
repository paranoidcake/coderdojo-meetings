extern crate web_view;
extern crate types;
use web_view::*;

fn main() {
    let html_content = include_str!("../dist/bundle.html");
    let mut rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async {
        web_view::builder()
        .title("My Project")
        .content(Content::Html(html_content))
        .size(320, 480)
        .resizable(false)
        .debug(true)
        .user_data(())
        .invoke_handler(|webview: &mut WebView<()>, arg: &str| {
            handle_message(webview.handle(), arg.to_string());
            Ok(())
        })
        .run()
        .unwrap();
    });
}

extern crate serde_json;
extern crate serde_derive;
use serde_derive::*;
// use serde::Serialize;
// use serde::de::DeserializeOwned;

#[derive(Serialize, Deserialize, std::fmt::Debug)]
pub struct Message<T> {
    subscription_id: String,
    message_id: String,
    inner: T
}

fn handle_message<
    // OUT: DeserializeOwned + Serialize + Send,
    // H: Fn(Request) -> std::option::Option<OUT> + Send
>(handle: Handle<()>, arg: String, /* handler: H */) {
    tokio::spawn(
        async move {
            let recieved: Message<types::webview::Request> = serde_json::from_str(&arg).unwrap();

            // TODO: Get this to be passed into the function without needing a static lifetime
            // let output = handler(received.inner)

            let output: Option<types::webview::Return> = {
                use types::webview::Request::*;
                use types::webview::*;

                match &recieved.inner {
                    Init => {
                        None
                    }
                    Log { text } => {
                        println!("{}", text);
                        None
                    }
                    Increment { number } => {
                        Some(Return::Increment {
                            number: *number + 1
                        })
                    }
                    DelayedIncrement { number } => {
                        std::thread::sleep(std::time::Duration::from_millis(1000));
                        Some(Return::DelayedIncrement {
                            number: *number + 1
                        })
                    }
                    ToUpperCase { text } => {
                        let text: &String = text;
                        Some(Return::ToUpperCase {
                            text: text.to_string().to_ascii_uppercase()
                        })
                    }
                    Test => {
                        None
                    }
                }
            };

            if let Some(response) = output {
                handle.dispatch(move | webview | {
                    let sending = Message {
                        subscription_id: recieved.subscription_id,
                        message_id: recieved.message_id,
                        inner: serde_json::to_string(&response).unwrap()
                    };

                    let eval_script = format!(
                        r#"document.dispatchEvent(
                            new CustomEvent("{event_name}", {{ detail: {{ messageId: {message_id:?}, inner: {content} }} }})
                        );"#,
                        event_name = sending.subscription_id,
                        message_id = sending.message_id,
                        content = sending.inner
                    );

                    webview.eval(&eval_script)
                }).expect("Failed to send response");
            }
        }
    );
}