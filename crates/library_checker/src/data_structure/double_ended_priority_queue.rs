use competitive::data_structure::{MinMaxHeap, OfflineDoubleEndedPriorityQueue};
use competitive::prelude::*;

#[verify::library_checker("double_ended_priority_queue", include("MinMaxHeap"))]
pub fn double_ended_priority_queue(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, values: [i32; iter n]);
    let mut queue: MinMaxHeap<_> = values.collect();
    for _ in 0..q {
        sc!(kind: u8);
        match kind {
            0 => {
                sc!(x: i32);
                queue.push(x);
            }
            1 => pp!(queue.pop_min().unwrap()),
            _ => pp!(queue.pop_max().unwrap()),
        }
    }
}

#[verify::library_checker(
    "double_ended_priority_queue",
    include("OfflineDoubleEndedPriorityQueue")
)]
pub fn double_ended_priority_queue_offline(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, values: [i32; iter n]);
    let mut builder = OfflineDoubleEndedPriorityQueue::builder();
    for value in values {
        builder.insert(value);
    }
    let mut queries = Vec::with_capacity(q);
    for _ in 0..q {
        sc!(kind: u8);
        let handle = if kind == 0 {
            sc!(x: i32);
            builder.reserve(x)
        } else {
            0
        };
        queries.push((kind, handle));
    }
    let mut queue = builder.build_radix();
    for (kind, handle) in queries {
        match kind {
            0 => queue.push(handle),
            1 => pp!(queue.pop_min().unwrap()),
            _ => pp!(queue.pop_max().unwrap()),
        }
    }
}
