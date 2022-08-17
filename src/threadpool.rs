use std::thread::{JoinHandle, spawn};
use std::sync::{Arc, Mutex, mpsc};

type Work = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Work),
    Terminate
}

/// This holds the instance of a thread.
#[derive(Debug)]
struct Worker {
    pub _id:    u32,
    pub handle: Option<JoinHandle<()>>
}

impl Worker {
    pub fn new(id: u32, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Self {
        let thread = spawn(move || {
            loop {

                // Wait for the main thread to give us some work to do
                let message = receiver.lock().unwrap().recv().unwrap();

                // If the threadpool tells us to terminate, the we should break
                // out of our loop and terminate this thread
                let job = match message {
                    Message::NewJob(job) => job,
                    Message::Terminate   => {
                        println!("[+] Terminating Thread {}", id);
                        break;
                    },
                };

                // println!("[+] Executing on thread {}", id);

                // Do the work
                job();
            }
        });
       
        Worker {
            _id:     id,
            handle: Some(thread),
        }
    }
}

#[derive(Debug)]
pub struct Threadpool {
    workers:        Vec<Worker>,
    transmitter:    mpsc::Sender<Message>,
}

impl Threadpool {

    /// Create a new threadpool with `num` threads.
    pub fn new(num: u8) -> Self {

        // Create a list to hold the threads that we will be spawning
        let mut workers = Vec::<Worker>::with_capacity(num as usize);

        // Create a message passing channel that we can use to communicate
        // between the main thread and the worker threads. This is the channel
        // that will be used to send the work from the main thread to the worker
        // thread.
        let (tx, rx) = mpsc::channel();

        // The channel that was created above is multiple producer and single
        // consumer. We are going to wrap the consumer end in a mutex and then
        // share it with all the threads that we will spawn. The thread will
        // wait on the mutex and when it gets access to the consumer then we are
        // assured that this is the only thread that has access to the work that
        // the main thread sent it.
        let receiver = Arc::new(Mutex::new(rx));

        // Create the workers now
        for i in 0..num {
            let receiver = receiver.clone();
            let worker = Worker::new(i as u32, receiver);
            workers.push(worker);
        }

        Threadpool {
            workers:        workers,
            transmitter:    tx
        }
    }

    /// Receives a closure and then sends it to the threads. The first one to be
    /// free can claim the closure and then execute it on that thread.
    pub fn execute<F>(&self, func: F)
        where F: FnOnce() + Send + 'static {

        // Wrap the closure in a Box prior to sending it to the thread.
        let func = Box::new(func);

        // Now send this to all the threads. Keep in mind that this is a single
        // consumer channel and all the threads have the same reference to the
        // consumer end. The mutex that we use in the thread ensures that only
        // one of the threads gets access to this closure
        // self.transmitter.send(func).unwrap();
        self.transmitter.send(Message::NewJob(func)).unwrap();
    }
}

impl Drop for Threadpool {

    fn drop(&mut self) {

        // Send the terminate signal to all the threads
        for _ in &self.workers {
            self.transmitter.send(Message::Terminate).unwrap();
        }

        // Now wait on each individual thread to die and replace the thread
        // handle with a None.
        for worker in &mut self.workers {
            let handle = worker.handle.take();
            if let Some(handle) = handle {
                handle.join().unwrap();
            }
        }
    }
}
