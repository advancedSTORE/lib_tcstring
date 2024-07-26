let
  rust-overlay = builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz";
  pkgs = import <nixpkgs> {
    overlays = [(import rust-overlay)];
  };
  toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
in
  pkgs.mkShell {
    packages = [
      toolchain
    ];

    nativeBuildInputs = [ pkgs.pkg-config ];
    buildInputs = [ pkgs.openssl_3_3 ];

    RUST_SRC_PATH = pkgs.rust-bin.stable.latest.rust-src;
    LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [ pkgs.openssl_3_3 ];
  }
