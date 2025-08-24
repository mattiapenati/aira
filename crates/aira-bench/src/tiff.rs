use std::{io::Read, iter::repeat_with};

use aira::tiff::{ByteOrder, predictor::IntegerPredictorReader};
use claims::*;
use criterion::{BenchmarkId, Criterion, SamplingMode, Throughput};

const KB: usize = 1024;
const MB: usize = 1024 * KB;
const LENS: &[usize] = &[KB, 4 * KB, 16 * KB, 256 * KB, 16 * MB];

pub fn bench(c: &mut Criterion) {
    horizontal_differencing_u8(c);
}

fn horizontal_differencing_u8(c: &mut Criterion) {
    let mut group = c.benchmark_group("tiff/horizontal_differencing_u8");
    group.sampling_mode(SamplingMode::Flat).sample_size(20);

    for size in LENS {
        let src = repeat_with(|| fastrand::u8(..))
            .take(*size)
            .collect::<Vec<_>>();
        let mut dst = vec![0u8; *size];
        let endian = ByteOrder::native();

        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut reader =
                    assert_ok!(IntegerPredictorReader::new(&src[..], endian, 1024, 1, 1));
                assert_ok!(reader.read_exact(&mut dst[..size]));
            });
        });
    }
    group.finish();
}
