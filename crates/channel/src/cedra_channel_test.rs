// Copyright © Cedra Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::{cedra_channel, cedra_channel::ElementStatus, message_queues::QueueStyle};
use cedra_types::account_address::AccountAddress;
use futures::{
    channel::oneshot,
    executor::block_on,
    future::{join, FutureExt},
    stream::{FusedStream, StreamExt},
};
use std::time::Duration;
use tokio::{runtime::Runtime, time::sleep};

#[test]
fn test_send_recv_order() {
    let (sender, mut receiver) = cedra_channel::new(QueueStyle::FIFO, 10, None);
    sender.push(0, 0).unwrap();
    sender.push(0, 1).unwrap();
    sender.push(0, 2).unwrap();
    sender.push(0, 3).unwrap();
    let task = async move {
        // Ensure that messages are received in order
        assert_eq!(receiver.select_next_some().await, 0);
        assert_eq!(receiver.select_next_some().await, 1);
        assert_eq!(receiver.select_next_some().await, 2);
        assert_eq!(receiver.select_next_some().await, 3);
        // Ensures that there is no other value which is ready
        assert_eq!(receiver.select_next_some().now_or_never(), None);
    };
    block_on(task);
}

#[test]
fn test_empty() {
    let (_, mut receiver) = cedra_channel::new::<u8, u8>(QueueStyle::FIFO, 10, None);
    // Ensures that there is no other value which is ready
    assert_eq!(receiver.select_next_some().now_or_never(), None);
}

#[test]
fn test_waker() {
    let (sender, mut receiver) = cedra_channel::new(QueueStyle::FIFO, 10, None);
    // Ensures that there is no other value which is ready
    assert_eq!(receiver.select_next_some().now_or_never(), None);
    let f1 = async move {
        assert_eq!(receiver.select_next_some().await, 0);
        assert_eq!(receiver.select_next_some().await, 1);
        assert_eq!(receiver.select_next_some().await, 2);
    };
    let f2 = async {
        sleep(Duration::from_millis(100)).await;
        sender.push(0, 0).unwrap();
        sleep(Duration::from_millis(100)).await;
        sender.push(0, 1).unwrap();
        sleep(Duration::from_millis(100)).await;
        sender.push(0, 2).unwrap();
    };
    let rt = Runtime::new().unwrap();
    rt.block_on(join(f1, f2));
}

#[test]
fn test_sender_clone() {
    let (sender, mut receiver) = cedra_channel::new(QueueStyle::FIFO, 5, None);
    // Ensures that there is no other value which is ready
    assert_eq!(receiver.select_next_some().now_or_never(), None);

    let _sender_clone = sender.clone();

    let f1 = async move {
        sender.push(0, 0).unwrap();
        sender.push(0, 1).unwrap();
    };

    let f2 = async move {
        assert_eq!(receiver.select_next_some().await, 0);
        assert_eq!(receiver.select_next_some().await, 1);
        assert_eq!(receiver.select_next_some().now_or_never(), None);

        // receiver should not think stream is terminated, since
        // sender_clone is not dropped yet (sender is dropped at this point)
        assert!(!receiver.is_terminated());
    };

    block_on(f1);
    block_on(f2);
}

fn test_multiple_validators_helper(
    queue_style: QueueStyle,
    num_messages_per_validator: usize,
    expected_last_message: usize,
) {
    let (sender, mut receiver) = cedra_channel::new(queue_style, 1, None);
    let num_validators = 128;
    for message in 0..num_messages_per_validator {
        for validator in 0..num_validators {
            sender
                .push(
                    AccountAddress::new([validator as u8; AccountAddress::LENGTH]),
                    (validator, message),
                )
                .unwrap();
        }
    }
    block_on(async {
        for i in 0..num_validators {
            assert_eq!(
                receiver.select_next_some().await,
                (i, expected_last_message)
            );
        }
    });
    assert_eq!(receiver.select_next_some().now_or_never(), None);
}

#[test]
fn test_multiple_validators_fifo() {
    test_multiple_validators_helper(QueueStyle::FIFO, 1024, 0);
}

#[test]
fn test_multiple_validators_lifo() {
    test_multiple_validators_helper(QueueStyle::LIFO, 1024, 1023);
}

#[test]
fn test_feedback_on_drop() {
    let (sender, mut receiver) = cedra_channel::new(QueueStyle::FIFO, 3, None);
    sender.push(0, 'a').unwrap();
    sender.push(0, 'b').unwrap();
    let (c_status_tx, c_status_rx) = oneshot::channel();
    sender
        .push_with_feedback(0, 'c', Some(c_status_tx))
        .unwrap();
    let (d_status_tx, d_status_rx) = oneshot::channel();
    sender
        .push_with_feedback(0, 'd', Some(d_status_tx))
        .unwrap();
    let task = async move {
        // Ensure that the remaining messages are received in order
        assert_eq!(receiver.select_next_some().await, 'a');
        assert_eq!(receiver.select_next_some().await, 'b');
        assert_eq!(receiver.select_next_some().await, 'c');
        // Ensure that we receive confirmation about 'd' being dropped and 'c' being delivered.
        assert_eq!(ElementStatus::Dropped('d'), d_status_rx.await.unwrap());
        assert_eq!(ElementStatus::Dequeued, c_status_rx.await.unwrap());
        // Ensures that there is no other value which is ready
        assert_eq!(receiver.select_next_some().now_or_never(), None);
    };
    block_on(task);
}
