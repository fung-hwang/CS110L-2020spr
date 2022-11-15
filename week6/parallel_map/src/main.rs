use crossbeam_channel;
use std::{thread, time};

fn parallel_map<T, U, F>(mut input_vec: Vec<T>, num_threads: usize, f: F) -> Vec<U>
// Is "mut" in "mut input_vec: Vec<T>" necessary?
where
    F: FnOnce(T) -> U + Send + Copy + 'static,
    T: Send + 'static,
    U: Send + 'static + Default,
{
    let mut output_vec: Vec<U> = Vec::with_capacity(input_vec.len());
    output_vec.resize_with(input_vec.len(), Default::default); // need to init the elements

    let mut handles = Vec::new();
    let (s1, r1) = crossbeam_channel::unbounded(); // channel_1: thread main -> thread n
    let (s2, r2) = crossbeam_channel::unbounded(); // channel_2: thread n -> thread main
    for (idx, val) in input_vec.into_iter().enumerate() {
        // send val and its index of input_vec to thread n by channel_1
        // Type: (idx, val)
        s1.send((idx, val)).unwrap();
    }
    drop(s1);
    // thread n
    for _ in 0..num_threads {
        let r1 = r1.clone();
        let s2 = s2.clone();
        let handle = thread::spawn(move || {
            // receive (idx, val) from thread main, execute f
            // and send (idx, f(val)) to thread main by channel_2
            while let Ok((idx, val)) = r1.recv() {
                s2.send((idx, f(val))).unwrap();
            }
        });
        handles.push(handle);
    }

    drop(s2); // important!

    // receive (idx, f(val)) from thread n by channel_2
    while let Ok((idx, val)) = r2.recv() {
        *output_vec.get_mut(idx).unwrap() = val;
    }

    for handle in handles {
        handle.join().expect("join thread failed");
    }

    output_vec
}

fn main() {
    let v = vec![6, 7, 8, 9, 10, 1, 2, 3, 4, 5, 12, 18, 11, 5, 20];
    let squares = parallel_map(v, 10, |num| {
        println!("{} squared is {}", num, num * num);
        thread::sleep(time::Duration::from_millis(500));
        num * num
    });
    println!("squares: {:?}", squares);
}
