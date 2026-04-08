use std::error::Error;
use std::sync::{Arc, Mutex, mpsc, mpsc::Receiver};
use std::thread;
use std::thread::JoinHandle;

pub struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,
}

type ReceiverEnd = Arc<Mutex<Receiver<Job>>>;

impl Worker {
    pub fn new(id: usize, rx: ReceiverEnd) -> Self {
        let thread = thread::spawn(move || {
            loop {
                let job = {
                    let rx = rx.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                    rx.recv()
                };

                match job {
                    Ok(job) => job(),
                    Err(_) => {
                        eprintln!("Worker {} shutting down.", id);
                        break;
                    }
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    tx: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    fn new(size: usize) -> Result<Self, Box<dyn Error>> {
        let (tx, rx) = mpsc::channel();
        let rx = Arc::new(Mutex::new(rx));

        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&rx)));
        }

        Ok(ThreadPool {
            workers,
            tx: Some(tx),
        })
    }

    pub fn build(size: usize) -> Result<Self, Box<dyn Error>> {
        if size > 0 {
            Ok(ThreadPool::new(size)?)
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "size must be greater than 0",
            )))
        }
    }

    pub fn execute<F>(&self, f: F) -> Result<(), Box<dyn Error>>
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.tx
            .as_ref()
            .ok_or("thread pool has been shut down")?
            .send(job)
            .map_err(|e| e.into())
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.tx.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
