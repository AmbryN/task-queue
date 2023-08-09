use axum::{Router, routing::{get, post}, Json, extract::State};
use axum_macros::debug_handler;
use serde::{Serialize, Deserialize};
use std::{net::SocketAddr, sync::{Arc, RwLock}};

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let shared_queue = SharedQueue::new(RwLock::new(Queue::new()));

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/tasks", get(tasks))
        .route("/tasks", post(create_task))
        .with_state(Arc::clone(&shared_queue));

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[debug_handler]
async fn tasks(state: State<SharedQueue>) -> Json<Vec<Task>> {
    Json(state.read().unwrap().tasks.clone())
}

#[debug_handler]
async fn create_task(state: State<SharedQueue>, Json(payload): Json<Task>) ->  Json<Task> {
    let task = Task {
        duration: payload.duration
    };
    state.write().unwrap().enqueue(task);
    Json(task)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct Task {
    duration: usize
}

#[derive(Debug, Default)]
struct Queue {
    tasks: Vec<Task>
}

impl Queue {
    fn new() -> Queue {
        Queue {
            tasks: vec![
                Task {
                    duration: 3
                },
                Task {
                    duration: 5
                }
            ]
        }
    }

    fn enqueue(&mut self, task: Task) {
        self.tasks.push(task);
    }
}

type SharedQueue = Arc<RwLock<Queue>>;

