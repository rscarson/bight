bench:
	mkdir bight/benchmark_results/latest
	cargo +nightly bench -p bight -F multi-thread -- -Z unstable-options --format=json > bight/benchmark_results/latest/multi-thread.json;  
	cargo +nightly bench -p bight -F multi-thread > bight/benchmark_results/latest/multi-thread.txt;
	cargo +nightly bench -p bight -- -Z unstable-options --format=json > bight/benchmark_results/latest/single-thread.json;
	cargo +nightly bench -p bight  > bight/benchmark_results/latest/single-thread.txt;
