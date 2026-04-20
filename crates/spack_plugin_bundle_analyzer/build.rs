use std::{env, fs, path::PathBuf};

fn main() {
  println!("cargo:rerun-if-env-changed=SPACK_BUNDLE_VIEWER_HTML");

  let asset_path = env::var_os("SPACK_BUNDLE_VIEWER_HTML").unwrap_or_else(|| {
    panic!(
      "SPACK_BUNDLE_VIEWER_HTML must point to a generated bundle-viewer HTML asset. Run `pnpm run bundle-viewer:build-asset` or build through the JS binding script."
    )
  });
  let asset_path = PathBuf::from(asset_path);
  let asset_path = fs::canonicalize(&asset_path).unwrap_or_else(|error| {
    panic!(
      "Failed to resolve SPACK_BUNDLE_VIEWER_HTML at {}: {error}",
      asset_path.display()
    )
  });

  if !asset_path.is_file() {
    panic!(
      "SPACK_BUNDLE_VIEWER_HTML must reference a file, got {}",
      asset_path.display()
    );
  }

  println!("cargo:rerun-if-changed={}", asset_path.display());

  let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set by Cargo"));
  let output_path = out_dir.join("bundle-viewer.html");
  fs::copy(&asset_path, &output_path).unwrap_or_else(|error| {
    panic!(
      "Failed to copy bundle viewer asset from {} to {}: {error}",
      asset_path.display(),
      output_path.display()
    )
  });
}
