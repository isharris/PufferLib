use std::env;

fn main() {
    println!("cargo:rerun-if-env-changed=CUDA_HOME");
    println!("cargo:rerun-if-env-changed=ROCM_HOME");
    println!("cargo:rerun-if-changed=cuda/puff_advantage.cu");
    println!("cargo:rustc-check-cfg=cfg(puffer_cuda)");

    let cuda_home = env::var("CUDA_HOME").ok();
    if cuda_home.is_none() {
        return;
    }

    let mut build = cc::Build::new();
    build.cuda(true).file("cuda/puff_advantage.cu");

    // Make every wheel-build attempt visible; users can opt out with NO_CUDA=1.
    if env::var("NO_CUDA").ok().as_deref() == Some("1") {
        return;
    }

    if let Err(e) = std::panic::catch_unwind(|| build.compile("puff_advantage_cuda")) {
        eprintln!("warning: CUDA compilation failed ({:?}); building without CUDA", e);
        return;
    }

    let cuda_home = cuda_home.unwrap();
    println!("cargo:rustc-link-search=native={}/lib64", cuda_home);
    println!("cargo:rustc-link-search=native={}/lib", cuda_home);
    println!("cargo:rustc-link-lib=dylib=cudart");
    println!("cargo:rustc-cfg=puffer_cuda");
}
