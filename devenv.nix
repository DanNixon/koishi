{pkgs, ...}: {
  packages = with pkgs; [
    # Rust toolchain
    rustup

    # Code formatting tools
    treefmt
    alejandra
    mdl

    # Rust dependency linting
    cargo-deny
  ];
}
