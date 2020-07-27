use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lib_tcstring::TCModelV1;
use std::convert::TryFrom;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("TCString V1 (small bitfield section variant)", |b| {
        b.iter(|| TCModelV1::try_from(black_box("BOEFEAyOEFEAyAHABDENAI4AAAB9vABAASA")))
    });
    c.bench_function("TCString V1 (large bitfield section variant)", |b| {
        b.iter(|| TCModelV1::try_from(black_box("BOyRMJVO2IaNjAKAiBENDR-AAAAwxrv7_77e_9f-_f__9uj3Gr_v_f__3mccL5tv3hv7v6_7fi_-1nV4u_1tft9ydk1-5YtDzto507iakiPHmqNeb1n_mz1eZpRP58E09j53z7Ew_v8_v-b7BCPN_Y3v-8K96lGA")))
    });
    c.bench_function("TCString V1 (large range section variant)", |b| {
        b.iter(|| TCModelV1::try_from(black_box("BO2IUIWO2IUIWB9ABADEDR-AAAAwyABgACBhgA")))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
