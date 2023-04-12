use std::sync::Arc;

use anyhow::Context;
use axum::{
    extract::{Query, State},
    response::{
        sse::{self, Sse},
        IntoResponse,
    },
    routing::MethodRouter,
    Extension,
};
use futures::{
    stream::{self, BoxStream},
    Stream, StreamExt,
};

use crate::{query::parser, Application};

use super::{Error, Result};

mod answer_api;
mod prompts;

#[derive(Default)]
pub struct RouteState {
    conversations: scc::HashMap<ConversationId, Conversation>,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Params {
    pub q: String,
    pub thread_id: String,
    pub user_id: String,
}

pub(super) fn endpoint<S>() -> MethodRouter<S> {
    let state = Arc::new(RouteState::default());
    axum::routing::post(handle).with_state(state)
}

pub(super) async fn handle(
    Query(params): Query<Params>,
    State(state): State<Arc<RouteState>>,
    Extension(app): Extension<Application>,
) -> Result<impl IntoResponse> {
    dbg!("handling");
    let conversation_id = ConversationId {
        user_id: params.user_id,
        thread_id: params.thread_id,
    };

    let mut conversation: Conversation = state
        .conversations
        .read_async(&conversation_id, |_k, v| v.clone())
        .await
        .unwrap_or_default();

    let answer_client = answer_api::Client::new(&app.config.answer_api_url);

    let q = params.q;
    let stream = async_stream::try_stream! {
        let mut action = Action::Query(q);

        loop {
            let (next, upds) = conversation.step(&answer_client, action).await?;

            for await upd in upds {
                yield upd;
            }

            match next {
                Some(a) => action = a,
                None => break,
            }
        }
    };

    let stream = stream
        .map(|upd: anyhow::Result<_>| {
            sse::Event::default().json_data(&upd.map_err(|e| e.to_string()))
        })
        .chain(futures::stream::once(async {
            Ok(sse::Event::default().data("[DONE]"))
        }));

    Ok(Sse::new(stream))
}

#[derive(Hash, PartialEq, Eq)]
struct ConversationId {
    thread_id: String,
    user_id: String,
}

#[derive(Clone)]
struct Conversation {
    history: Vec<answer_api::api::Message>,
    path_aliases: Vec<String>,
}

impl Default for Conversation {
    fn default() -> Self {
        Self {
            history: vec![
                answer_api::api::Message::system(prompts::SYSTEM),
                answer_api::api::Message::assistant(
                    r#"["ask","Hi there, what can I help you with?"]"#,
                ),
            ],
            path_aliases: Vec::new(),
        }
    }
}

impl Conversation {
    fn path_alias(&mut self, path: &str) -> usize {
        if let Some(i) = self.path_aliases.iter().position(|p| *p == path) {
            i
        } else {
            let i = self.path_aliases.len();
            self.path_aliases.push(path.to_owned());
            i
        }
    }

    async fn step(
        &mut self,
        answer_client: &answer_api::Client,
        action: Action,
    ) -> anyhow::Result<(Option<Action>, BoxStream<'static, Update>)> {
        dbg!(&action);

        let question = match action {
            Action::Query(s) => {
                let query = parser::parse_nl(&s)?;
                let question = query
                    .target
                    .context("query was empty")?
                    .as_plain()
                    .context("user query was not plain text")?
                    .clone()
                    .into_owned();

                question
            }

            Action::Prompt(s) => {
                return Ok((
                    None,
                    Box::pin(stream::once(async move { Update::Prompt(s) })),
                ));
            }

            Action::Path(search) => {
                use super::query::QueryResult;

                let mut client = reqwest::Client::new();
                let res = client
                    .get("http://127.0.0.1:7878/api/q")
                    .query(&[("q", format!("path:{search}"))])
                    .send()
                    .await?;

                let mut body = res.json::<crate::webserver::query::QueryResponse>().await?;

                let paths = Some("§alias, path".to_owned())
                    .into_iter()
                    .chain(
                        body.data
                            .into_iter()
                            .map(|d| match d {
                                QueryResult::FileResult(d) => d,
                                _ => {
                                    todo!("bad result type {}", serde_json::to_string(&d).unwrap())
                                }
                            })
                            .map(|d| d.relative_path.text)
                            .map(|path| format!("{}, {path}", self.path_alias(&path))),
                    )
                    .collect::<Vec<_>>()
                    .join("\n");

                paths
            }

            Action::Answer(answer) => {
                return Ok((
                    Some(Action::Prompt(
                        "Is there anything else I can help with?".to_owned(),
                    )),
                    Box::pin(stream::once(async move { Update::Answer(answer) })),
                ));
            }

            Action::Code(query) => {
                let mut client = reqwest::Client::new();
                let res = client
                    .get("http://127.0.0.1:7878/api/semantic/chunks")
                    .query(&[("query", query), ("limit", "10".to_owned())])
                    .send()
                    .await?;

                let mut body = res
                    .json::<crate::webserver::semantic::SemanticResponse>()
                    .await?;

                let chunks = body
                    .chunks
                    .into_iter()
                    .map(|chunk| {
                        let relative_path = chunk["relative_path"].as_str().unwrap();
                        serde_json::json!({
                            "path": relative_path,
                            "§ALIAS": self.path_alias(relative_path),
                            "snippet": chunk["snippet"],
                            "start": chunk["start_line"].as_str().unwrap().parse::<u32>().unwrap(),
                            "end": chunk["end_line"].as_str().unwrap().parse::<u32>().unwrap(),
                        })
                    })
                    .collect::<Vec<_>>();

                serde_json::to_string(&chunks).unwrap()
            }

            Action::File(path) => {
                use super::query::QueryResult;

                let mut client = reqwest::Client::new();
                let res = client
                    .get("http://127.0.0.1:7878/api/q")
                    .query(&[("q", format!("open:true repo:answer-api path:{path}"))])
                    .send()
                    .await?;

                let mut body = res.json::<crate::webserver::query::QueryResponse>().await?;

                let contents = body
                    .data
                    .into_iter()
                    .map(|d| match d {
                        QueryResult::File(f) => f,
                        _ => {
                            todo!("bad result type {}", serde_json::to_string(&d).unwrap())
                        }
                    })
                    .map(|f| f.contents)
                    .next()
                    .expect("failed to open file");

                contents
            }
        };

        self.history.push(answer_api::api::Message::user(
            &(question + "\n\nAnswer only with a JSON action."),
        ));

        let res = {
            let mut stream = Box::pin(answer_client.chat(self.history.clone()).await?);
            let mut buf = String::new();
            while let Some(r) = stream.next().await {
                buf += &r?;
            }
            buf
        };

        self.history.push(answer_api::api::Message::assistant(&res));

        dbg!(&res);

        let action: Action = serde_json::from_str(&res)?;
        let updates = action.update();

        Ok((Some(action), Box::pin(updates)))
    }

    fn trimmed_hist(&self) -> Vec<answer_api::api::Message> {
        // TODO
        self.history.clone()
    }
}

#[derive(Debug)]
enum Action {
    /// A user-provided query.
    Query(String),
    Prompt(String),
    Path(String),
    Answer(String),
    Code(String),
    File(String),
}

impl Action {
    fn update(&self) -> impl Stream<Item = Update> {
        let upd = match self {
            Self::Prompt(..) => unreachable!("this variant should have led to an early return"),
            Self::Query(q) => unreachable!("the model tried to create a user query {q}"),
            Self::Code(..) => Update::SearchingFiles,
            Self::Path(..) => Update::SearchingFiles,
            Self::Answer(..) => Update::Answering,
            Self::File(..) => Update::LoadingFiles,
        };

        stream::once(async move { upd })
    }
}

impl<'de> serde::Deserialize<'de> for Action {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let (action, text) = <(String, String)>::deserialize(deserializer)?;

        Ok(match action.as_str() {
            "ask" => Self::Prompt(text),
            "path" => Self::Path(text),
            "answer" => Self::Answer(text),
            "code" => Self::Code(text),
            "file" => Self::File(text),
            other => todo!("unknown action [{other}, {text}]"),
        })
    }
}

#[derive(serde::Serialize)]
enum Update {
    Prompt(String),
    Answer(String),
    SearchingFiles,
    Answering,
    LoadingFiles,
}
