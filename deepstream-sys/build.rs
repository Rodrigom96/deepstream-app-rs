fn main() {
    println!("cargo:rustc-link-search=/opt/nvidia/deepstream/deepstream/lib");
    println!("cargo:rustc-link-lib=nvdsgst_helper");
    //println!("cargo:rustc-link-lib=nvdsgst_meta");
    //println!("cargo:rustc-link-lib=nvds_meta");
    //println!("cargo:rustc-link-lib=nvds_infer");
}
