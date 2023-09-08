use axum::{
    response::Html,
    routing::{get, post},
    Form, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(root))
        .route("/inclauserate", post(inclauserate_handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3200));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> Html<&'static str> {
    let my_html = include_str!("./web.html");
    Html(&my_html)
}

#[derive(Deserialize, Serialize, Clone, Debug)]
struct InclauserateRequest {
    string_list: String,
    var_name: String,
}

#[axum_macros::debug_handler]
async fn inclauserate_handler(Form(inc): Form<InclauserateRequest>) -> String {
    inclauserate(&inc.string_list, &inc.var_name)
}

fn inclauserate(s: &String, var_name: &String) -> String {
    let mut stringy: Vec<String> = s
        .split('\n')
        .map(|x| x.trim_end())
        .filter(|x| x.len() > 0)
        .map(|x| format!("\"{}\",", x))
        .collect();
    stringy.dedup();
    let entries = stringy.len();
    if entries == 0 {
        return format!("{} IN ()", &var_name);
    }
    let mut in_str = format!("{}", &var_name);
    let mut fmt_string: String = "IN".to_string();
    let chunks = entries / 10000 + 1;
    for chunk in 0..chunks {
        if chunk == 1 {
            fmt_string = format!(" OR {} IN", &var_name);
        }
        let start = chunk * 10000;
        let end = std::cmp::min((chunk + 1) * 10000, stringy.len());
        let for_appending: String = stringy[start..end]
            .iter()
            .fold("".to_string(), |acc, f| acc + f);
        in_str = format!(
            "{} {} ({})",
            in_str,
            fmt_string,
            // trim the last comma off the end
            &for_appending[..for_appending.len() - 1]
        )
    }
    in_str
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn inclauseration() {
        let input = "123\n125".to_string();
        assert_eq!(
            inclauserate(&input, &format!("fake_news")),
            r#"fake_news IN ("123","125")"#
        )
    }
    #[test]
    fn twentyk() {
        let input = (0..24000).fold("".to_string(), |acc, idx| acc + "\n" + &idx.to_string());
        let ors = inclauserate(&input, &"fake_news".to_string())
            .split(" OR ")
            .collect::<Vec<_>>()
            .len();
        assert_eq!(ors, 3);
    }
    #[test]
    fn degenerate() {
        assert_eq!(inclauserate(&"".to_string(), &"F".to_string()), "F IN ()")
    }
    #[test]
    fn twentyk_dedupe() {
        let input = (0..24000).fold("".to_string(), |acc, _| acc + "123\n");
        assert_eq!(
            inclauserate(&input, &"fake_news".to_string()),
            "fake_news IN (\"123\")"
        );
    }
}
