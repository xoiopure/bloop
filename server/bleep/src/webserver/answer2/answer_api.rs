use anyhow::bail;
use futures::{Stream, StreamExt};
use reqwest_eventsource::EventSource;

pub mod api {
    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
    pub struct Message {
        pub role: String,
        pub content: String,
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
    pub struct Messages {
        pub messages: Vec<Message>,
    }

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    pub struct Request {
        pub messages: Messages,
        pub provider: Provider,
        pub max_tokens: Option<u32>,
        pub temperature: Option<f32>,
        #[serde(default)]
        pub extra_stop_sequences: Vec<String>,
    }

    #[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum Provider {
        OpenAi,
        Anthropic,
    }

    #[derive(thiserror::Error, Debug, serde::Deserialize)]
    pub enum Error {
        #[error("bad OpenAI request")]
        BadOpenAiRequest,

        #[error("incorrect configuration")]
        BadConfiguration,
    }

    pub type Result = std::result::Result<String, Error>;
}

impl api::Message {
    pub fn new(role: &str, content: &str) -> Self {
        Self {
            role: role.to_owned(),
            content: content.to_owned(),
        }
    }

    pub fn system(content: &str) -> Self {
        Self::new("system", content)
    }

    pub fn user(content: &str) -> Self {
        Self::new("user", content)
    }

    pub fn assistant(content: &str) -> Self {
        Self::new("assistant", content)
    }
}

pub struct Client {
    http: reqwest::Client,
    base_url: String,
    bearer_token: Option<String>,

    temperature: Option<f32>,
    max_tokens: Option<u32>,
    provider: api::Provider,
}

impl Client {
    pub fn new(base_url: &str) -> Self {
        Self::new_with_bearer(base_url, None)
    }

    pub fn new_with_bearer(base_url: &str, bearer_token: Option<&str>) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.to_owned(),
            bearer_token: bearer_token.map(str::to_owned),

            provider: api::Provider::OpenAi,
            temperature: None,
            max_tokens: None,
        }
    }

    pub fn temperature(mut self, temperature: Option<f32>) -> Self {
        self.temperature = temperature;
        self
    }

    pub fn max_tokens(mut self, max_tokens: Option<u32>) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    pub async fn chat(
        &self,
        messages: Vec<api::Message>,
    ) -> anyhow::Result<impl Stream<Item = anyhow::Result<String>>> {

        let mut event_source = Box::pin(
            EventSource::new({
                let mut builder = self
                    .http
                    .post(format!("{}/v1/q", self.base_url));

                if let Some(bearer) = &self.bearer_token {
                    builder = builder.bearer_auth(bearer);
                }

                builder.json(&api::Request {
                    messages: api::Messages { messages },
                    max_tokens: self.max_tokens,
                    temperature: self.temperature,
                    provider: self.provider,
                    extra_stop_sequences: vec![],
                })
            })
            // We don't have a `Stream` body so this can't fail.
            .expect("couldn't clone requestbuilder")
            // `reqwest_eventsource` returns an error to signify a stream end, instead of simply ending
            // the stream. So we catch the error here and close the stream.
            .take_while(|result| {
                let is_end = matches!(result, Err(reqwest_eventsource::Error::StreamEnded));
                async move { !is_end }
            }),
        );

        match event_source.next().await {
            Some(Ok(reqwest_eventsource::Event::Open)) => {}
            Some(Err(e)) => bail!("event source error: {:?}", e),
            _ => bail!("event source failed to open"),
        }

        Ok(event_source
            .filter_map(|result| async move {
                match result {
                    Ok(reqwest_eventsource::Event::Message(msg)) => Some(Ok(msg.data)),
                    Ok(reqwest_eventsource::Event::Open) => None,
                    Err(reqwest_eventsource::Error::StreamEnded) => None,
                    Err(e) => Some(Err(e)),
                }
            })
            .map(|result| match result {
                Ok(s) => Ok(serde_json::from_str::<api::Result>(&s)??),
                Err(e) => bail!("event source error {e:?}"),
            }))
    }
}
