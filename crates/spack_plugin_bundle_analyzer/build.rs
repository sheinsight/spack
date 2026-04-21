use std::{
  env, fs,
  path::{Path, PathBuf},
  process::Command,
};

fn main() {
  println!("cargo:rerun-if-env-changed=SPACK_BUNDLE_VIEWER_HTML");
  println!("cargo:rerun-if-changed=build.rs");

  let workspace_root = workspace_root();
  println!(
    "cargo:rerun-if-changed={}",
    workspace_root
      .join("scripts/build-bundle-viewer-asset.mjs")
      .display()
  );
  println!(
    "cargo:rerun-if-changed={}",
    workspace_root.join("packages/bundle-viewer").display()
  );

  let asset_path = resolve_bundle_viewer_asset(&workspace_root);
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

fn resolve_bundle_viewer_asset(workspace_root: &Path) -> PathBuf {
  if let Some(asset_path) = env::var_os("SPACK_BUNDLE_VIEWER_HTML") {
    return PathBuf::from(asset_path);
  }

  let generated_asset = workspace_root
    .join(".generated")
    .join("bundle-viewer")
    .join("bundle-viewer.html");

  if generated_asset.is_file() {
    return generated_asset;
  }

  if env::var_os("CI").is_some() {
    panic!(
      "SPACK_BUNDLE_VIEWER_HTML must be set in CI. Download the bundle-viewer-html artifact before building the binding."
    );
  }

  build_bundle_viewer_asset(workspace_root, &generated_asset);
  generated_asset
}

fn build_bundle_viewer_asset(workspace_root: &Path, generated_asset: &Path) {
  let status = Command::new("pnpm")
    .args([
      "--dir",
      workspace_root
        .to_str()
        .expect("workspace path is valid UTF-8"),
      "run",
      "bundle-viewer:build-asset",
    ])
    .status()
    .unwrap_or_else(|error| {
      panic!(
        "Failed to run `pnpm run bundle-viewer:build-asset`: {error}. \
If pnpm is unavailable, set SPACK_BUNDLE_VIEWER_HTML to a prebuilt asset."
      )
    });

  if !status.success() {
    panic!(
      "`pnpm run bundle-viewer:build-asset` exited with status {status}. \
If you already have a built asset, set SPACK_BUNDLE_VIEWER_HTML explicitly."
    );
  }

  if !generated_asset.is_file() {
    panic!(
      "Expected generated bundle viewer asset at {}, but it was not created.",
      generated_asset.display()
    );
  }
}

fn workspace_root() -> PathBuf {
  let manifest_dir =
    PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set by Cargo"));
  manifest_dir
    .ancestors()
    .nth(2)
    .expect("crate lives under <workspace>/crates/<name>")
    .to_path_buf()
}
