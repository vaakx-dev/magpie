use std::sync::LazyLock;

const ICO: &[u8] = include_bytes!("../../assets/magpie.ico");
const HEADER_BYTES: usize = 6;
const ENTRY_BYTES: usize = 16;
const COUNT_OFFSET: usize = 4;
const WIDTH_OFFSET: usize = 0;
const SIZE_OFFSET: usize = 8;
const DATA_OFFSET: usize = 12;
const FULL_SIZE_WIDTH: u16 = 256;

struct Entry {
    width: u16,
    start: usize,
    length: usize,
}

fn number(at: usize, bytes: usize) -> usize {
    ICO[at..at + bytes]
        .iter()
        .rev()
        .fold(0, |value, byte| value << 8 | usize::from(*byte))
}

fn entry(index: usize) -> Entry {
    let at = HEADER_BYTES + index * ENTRY_BYTES;
    let width = match ICO[at + WIDTH_OFFSET] {
        0 => FULL_SIZE_WIDTH,
        width => u16::from(width),
    };
    Entry {
        width,
        start: number(at + DATA_OFFSET, 4),
        length: number(at + SIZE_OFFSET, 4),
    }
}

fn largest() -> &'static [u8] {
    let largest = (0..number(COUNT_OFFSET, 2))
        .map(entry)
        .max_by_key(|entry| entry.width)
        .expect("the Magpie icon has images");
    &ICO[largest.start..largest.start + largest.length]
}

static PNG: LazyLock<&'static [u8]> = LazyLock::new(largest);

pub fn png() -> &'static [u8] {
    &PNG
}
