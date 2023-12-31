use serde::{Deserialize, Serialize};
use std::{
    sync::{mpsc::Sender, Arc, Mutex},
    time::Duration,
};

fn main() {
    let mut queue = Queue::new();

    // Initialiazing the channel for sending the tasks to and from the computation thread
    let (tx_task, rx_task) = std::sync::mpsc::channel::<Arc<Mutex<Task>>>();

    // Channel used to keep track of the current number of tasks being computed
    let (tx_nb, rx_nb) = std::sync::mpsc::channel::<u8>();

    // Creation of the computation thread
    std::thread::spawn(move || loop {
        println!("Début de la loop");
        // When a new task is sent down the channel
        let thread_task = rx_task.recv().unwrap();
        let thread_tx_nb = tx_nb.clone();

        println!("Lancement de la tâche");
        // Spawn a worker thread for the task
        std::thread::spawn(move || {
            // Acquire the lock on the task
            let mut task = thread_task.lock().unwrap();

            // Compute it synchronously
            let result = Some(task.compute() as usize);

            println!("{:?}", result);

            // Update the result
            task.result = result;

            // Tell the main thread a task is finished
            thread_tx_nb.send(1).unwrap();
        });
        println!("Fin de la loop");
    });

    let mut created = false;
    loop {
        let tx_task = tx_task.clone();

        // Start the processing of tasks in the queue
        process(&mut queue, tx_task);

        // Check for a finished task and update the number of currently running tasks
        if let Ok(val) = rx_nb.recv() {
            queue.current_number_tasks -= val;
        }

        if !created {
            create_task(
                &mut queue,
                Task {
                    duration: 2,
                    result: None,
                },
            );
            created = true;
        }
    }
}

fn create_task(queue: &mut Queue, task: Task) {
    queue.enqueue(task);
}

fn process(queue: &mut Queue, tx_task: Sender<Arc<Mutex<Task>>>) {
    while queue.current_number_tasks < queue.max_concurrent_tasks && queue.index < queue.tasks.len()
    {
        let thread_task = Arc::clone(&queue.tasks[queue.index]);

        tx_task.send(thread_task).unwrap();
        queue.current_number_tasks += 1;
        queue.index += 1;
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct Task {
    duration: usize,
    result: Option<usize>,
}

impl Task {
    fn compute(&self) -> u64 {
        let duration: u64 = self.duration as u64;

        std::thread::sleep(Duration::from_secs(duration));
        return duration;
    }
}

#[derive(Debug, Default)]
struct Queue {
    max_concurrent_tasks: u8,
    current_number_tasks: u8,
    index: usize,
    tasks: Vec<Arc<Mutex<Task>>>,
}

impl Queue {
    fn new() -> Queue {
        Queue {
            max_concurrent_tasks: 2,
            current_number_tasks: 0,
            index: 0,
            tasks: vec![
                Arc::new(Mutex::new(Task {
                    duration: 3,
                    result: None,
                })),
                Arc::new(Mutex::new(Task {
                    duration: 1,
                    result: None,
                })),
            ],
        }
    }

    fn enqueue(&mut self, task: Task) {
        self.tasks.push(Arc::new(Mutex::new(task)));
    }
}
