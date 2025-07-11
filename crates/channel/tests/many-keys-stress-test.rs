// Copyright © Cedra Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use cedra_channels::{cedra_channel, message_queues::QueueStyle};
use futures::{executor::block_on, stream::StreamExt};
use std::{
    io::{Cursor, Write},
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    thread,
    time::Duration,
};

/// A small benchmark/stress test that sends `num_msgs` for each `num_keys`. The
/// default arguments simulate many transient keys that just push a single message
/// and then never more. Without garbage collecting empty per-key-queues, the
/// program will eventually OOM.
#[derive(Debug)]
pub struct Args {
    num_keys: usize,
    num_msgs: usize,
    max_queue_size: usize,
}

pub fn run(args: Args) {
    // Simulates an AccountAddress/PeerId
    const KEY_SIZE_BYTES: usize = 16;

    // Simulates a (PeerManagerRequest, Option<Arc<_>>)
    const MSG_SIZE_BYTES: usize = 96;

    static NUM_PUSH: AtomicUsize = AtomicUsize::new(0);
    static NUM_POP: AtomicUsize = AtomicUsize::new(0);
    static IS_DONE: AtomicBool = AtomicBool::new(false);

    let (sender, mut receiver) = cedra_channel::new::<[u8; KEY_SIZE_BYTES], [u8; MSG_SIZE_BYTES]>(
        QueueStyle::FIFO,
        args.max_queue_size,
        None,
    );

    let sender_thread = thread::spawn(move || {
        for idx in 0..args.num_keys {
            let mut key = [0u8; KEY_SIZE_BYTES];
            let mut cursor = Cursor::new(&mut key[..]);
            cursor.write_all(&idx.to_le_bytes()).unwrap();

            for msg_idx in 0..args.num_msgs {
                let mut msg = [0u8; MSG_SIZE_BYTES];
                let mut cursor = Cursor::new(&mut msg[..]);
                cursor.write_all(&msg_idx.to_le_bytes()).unwrap();

                sender.push(key, msg).unwrap();
            }

            NUM_PUSH.fetch_add(1, Ordering::Relaxed);
        }
    });

    let logger_thread = thread::spawn(move || {
        while !IS_DONE.load(Ordering::Relaxed) {
            println!(
                "NUM_PUSH: {}, NUM_POP: {}",
                NUM_PUSH.load(Ordering::Relaxed),
                NUM_POP.load(Ordering::Relaxed),
            );
            thread::sleep(Duration::from_secs(1));
        }
    });

    // just drain messages
    let receiver_task = async move {
        while receiver.next().await.is_some() {
            NUM_POP.fetch_add(1, Ordering::Relaxed);
        }
    };

    block_on(receiver_task);
    sender_thread.join().unwrap();

    IS_DONE.store(true, Ordering::Relaxed);

    logger_thread.join().unwrap();
}

#[test]
fn test_many_keys_stress_test() {
    let args = Args {
        num_keys: 100,
        num_msgs: 1,
        max_queue_size: 10,
    };
    run(args);
}
