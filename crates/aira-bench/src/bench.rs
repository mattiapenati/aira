mod tiff;

fn main() {
    let mut c = criterion::Criterion::default()
        .configure_from_args()
        .without_plots()
        .configure_from_args()
        .warm_up_time(std::time::Duration::from_secs(1));
    tiff::bench(&mut c);
    c.final_summary();
}
