fn main() {
	cxx_build::bridge("src/bridge.rs")
		.flag_if_supported("-std=c++11")
		.compile("raptorcxx");
}
