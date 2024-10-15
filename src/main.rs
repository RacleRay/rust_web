use serde::Serialize;
use warp::Filter;

#[derive(Debug, Serialize)]
struct Question {
    id: QuestionId,
    title: String,
    content: String,
    tags: Option<Vec<String>>,
}

impl Question {
    fn new(id: QuestionId, title: String, content: String, tags: Option<Vec<String>>) -> Self {
        Question {
            id,
            title,
            content,
            tags,
        }
    }
}

#[derive(Debug, Serialize)]
struct QuestionId(String);

impl From<&str> for QuestionId {
    fn from(value: &str) -> Self {
        QuestionId(value.to_string())
    }
}

#[derive(Debug)]
struct InvalidId;
impl warp::reject::Reject for InvalidId {}

async fn get_questions() -> Result<impl warp::Reply, warp::Rejection> {
    let question = Question::new(
        "1".into(),
        "First Question".to_string(),
        "Content of the first question".to_string(),
        Some(vec!["rust".to_string(), "programming".to_string()]),
    );

    match question.id.0.parse::<u64>() {
        Ok(_) => Ok(warp::reply::json(&question)),
        Err(_) => Err(warp::reject::custom(InvalidId)),
    }
}

async fn return_error(r: warp::Rejection) -> Result<impl warp::Reply, warp::Rejection> {
    if let Some(error) = r.find::<warp::filters::cors::CorsForbidden>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            warp::http::StatusCode::FORBIDDEN,
        ))
    } else if let Some(_invalidid) = r.find::<InvalidId>() {
        Ok(warp::reply::with_status(
            "No valid ID given".to_string(),
            warp::http::StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else {
        Ok(warp::reply::with_status(
            "Route not found".to_string(),
            warp::http::StatusCode::NOT_FOUND,
        ))
    }
}

// curl -X OPTIONS localhost:3030/questions -H "Access-Control-Request-Method:PUT" -H "Access-Control-Request-Headers:content-type" -H "Origin:https://not-origin.io" --verbose
#[tokio::main]
async fn main() {
    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_methods(&[
            warp::http::Method::PUT,
            warp::http::Method::GET,
            warp::http::Method::POST,
            warp::http::Method::DELETE,
            warp::http::Method::OPTIONS,
        ]);

    let get_items = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and_then(get_questions)
        .recover(return_error);

    let routes = get_items.with(cors);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
