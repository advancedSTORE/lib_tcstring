use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lib_tcstring::TCModelV2;
use std::convert::TryFrom;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("TCString V2 (core only)", |b| {
        b.iter(|| TCModelV2::try_from(black_box("COvFyGBOvFyGBAbAAAENAPCAAOAAAAAAAAAAAEEUACCKAAA")))
    });
    c.bench_function("TCString V2 (core + disclosed vendors)", |b| {
        b.iter(|| TCModelV2::try_from(black_box("COvFyGBOvFyGBAbAAAENAPCAAOAAAAAAAAAAAEEUACCKAAA.IFoEUQQgAIQwgIwQABAEAAAAOIAACAIAAAAQAIAgEAACEAAAAAgAQBAAAAAAAGBAAgAAAAAAAFAAECAAAgAAQARAEQAAAAAJAAIAAgAAAYQEAAAQmAgBC3ZAYzUw")))
    });
    c.bench_function("TCString V2 (core + disclosed vendors + allowed vendors)", |b| {
        b.iter(|| TCModelV2::try_from(black_box("COw4XqLOw4XqLAAAAAENAXCAAAAAAAAAAAAAAAAAAAAA.YAAAAAAAAAAAAAAAAAA.QFukWSQgAIQwgI0QEByFAAAAeIAACAIgSAAQAIAgEQACEABAAAgAQFAEAIAAAGBAAgAAAAQAIFAAMCQAAgAAQiRAEQAAAAANAAIAAggAIYQFAAARmggBC3ZCYzU2yIA.IFukWSQgAIQwgI0QEByFAAAAeIAACAIgSAAQAIAgEQACEABAAAgAQFAEAIAAAGBAAgAAAAQAIFAAMCQAAgAAQiRAEQAAAAANAAIAAggAIYQFAAARmggBC3ZCYzU2yIA")))
    });
    c.bench_function("TCString V2 (core + publisher tc)", |b| {
        b.iter(|| {
            TCModelV2::try_from(black_box(
                "COw4XqLOw4XqLAAAAAENAXCAAP-gAAAfwIAAACngAI8AAA.cAEAPAAAC7gAHw4AAA",
            ))
        })
    });
    c.bench_function("TCString V2 (core + disclosed vendors + allowed vendors + publisher tc)", |b| {
        b.iter(|| TCModelV2::try_from(black_box("COw4XqLOw4XqLAAAAAENAXCf-v-gAAAfwIAAACngAI8AEFABgACAA4A.IAPPwAPrwA.QAPPwAPrwA.cAEAPAAAC7gAHw4AAA")))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
