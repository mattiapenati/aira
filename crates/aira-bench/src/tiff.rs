use criterion::Criterion;

pub fn bench(c: &mut Criterion) {
    predictor::u8(c);
    predictor::u16(c);
    predictor::u32(c);
    predictor::u64(c);
    predictor::f32(c);
    predictor::f64(c);
}

mod predictor {
    use std::{io::Read, iter::repeat_with};

    use aira::tiff::{
        ByteOrder,
        predictor::{FloatPredictorReader, IntPredictor},
    };
    use claims::*;
    use criterion::{
        BenchmarkGroup, BenchmarkId, Criterion, SamplingMode, Throughput, measurement::Measurement,
    };

    const KB: usize = 1024;
    const MB: usize = 1024 * KB;
    const LENS: &[usize] = &[KB, 4 * KB, 16 * KB, 256 * KB, 16 * MB];
    const SAMPLES: &[u16] = &[1, 2, 3, 4, 8];

    fn run_integer<T: Sized, M: Measurement>(
        mut group: BenchmarkGroup<'_, M>,
        byteorder: ByteOrder,
        samples: u16,
    ) {
        group.sampling_mode(SamplingMode::Flat).sample_size(10);

        for &size in LENS {
            let cols = (size / samples as usize / size_of::<T>()) as u32;
            let size = cols as usize * samples as usize * size_of::<T>();

            group.throughput(Throughput::Bytes(size as u64));
            group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
                let mut dst = vec![0u8; size];
                let mut predictor =
                    assert_ok!(IntPredictor::new(byteorder, samples, size_of::<T>() as u16,));

                b.iter(|| {
                    predictor.decode(&mut dst[..size]);
                    std::hint::black_box(&dst[..]);
                });
            });
        }

        group.finish();
    }

    fn run_float<T: Sized, M: Measurement>(mut group: BenchmarkGroup<'_, M>, samples: u16) {
        group.sampling_mode(SamplingMode::Flat).sample_size(10);

        for &size in LENS {
            let src = repeat_with(|| fastrand::u8(..))
                .take(size)
                .collect::<Vec<_>>();
            let mut dst = vec![0u8; size];

            let cols = (size / samples as usize / size_of::<T>()) as u32;
            let size = cols as usize * samples as usize * size_of::<T>();

            group.throughput(Throughput::Bytes(size as u64));
            group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
                b.iter(|| {
                    let mut reader = FloatPredictorReader::new(
                        &src[..size],
                        cols,
                        samples,
                        size_of::<T>() as u16,
                    );
                    assert_ok!(reader.read_exact(&mut dst[..size]));
                });
            });
        }

        group.finish();
    }

    struct DisplayByteOrder(ByteOrder);

    impl std::fmt::Display for DisplayByteOrder {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self.0 {
                ByteOrder::LittleEndian => write!(f, "little"),
                ByteOrder::BigEndian => write!(f, "big"),
            }
        }
    }

    pub fn u8(c: &mut Criterion) {
        for &samples in SAMPLES {
            let group_name = format!("tiff/predictor/u8/{samples}");
            let group = c.benchmark_group(group_name);
            run_integer::<u8, _>(group, ByteOrder::native(), samples);
        }
    }

    pub fn u16(c: &mut Criterion) {
        for &samples in SAMPLES {
            for byteorder in [ByteOrder::LittleEndian, ByteOrder::BigEndian] {
                let group_name = format!(
                    "tiff/predictor/u16/{samples}/{}",
                    DisplayByteOrder(byteorder)
                );
                let group = c.benchmark_group(group_name);
                run_integer::<u16, _>(group, byteorder, samples);
            }
        }
    }

    pub fn u32(c: &mut Criterion) {
        for &samples in SAMPLES {
            for byteorder in [ByteOrder::LittleEndian, ByteOrder::BigEndian] {
                let group_name = format!(
                    "tiff/predictor/u32/{samples}/{}",
                    DisplayByteOrder(byteorder)
                );
                let group = c.benchmark_group(group_name);
                run_integer::<u32, _>(group, byteorder, samples);
            }
        }
    }

    pub fn u64(c: &mut Criterion) {
        for &samples in SAMPLES {
            for byteorder in [ByteOrder::LittleEndian, ByteOrder::BigEndian] {
                let group_name = format!(
                    "tiff/predictor/u64/{samples}/{}",
                    DisplayByteOrder(byteorder)
                );
                let group = c.benchmark_group(group_name);
                run_integer::<u64, _>(group, byteorder, samples);
            }
        }
    }

    pub fn f32(c: &mut Criterion) {
        for &samples in SAMPLES {
            let group_name = format!("tiff/predictor/f32/{samples}",);
            let group = c.benchmark_group(group_name);
            run_float::<f32, _>(group, samples);
        }
    }

    pub fn f64(c: &mut Criterion) {
        for &samples in SAMPLES {
            let group_name = format!("tiff/predictor/f64/{samples}",);
            let group = c.benchmark_group(group_name);
            run_float::<f64, _>(group, samples);
        }
    }
}
