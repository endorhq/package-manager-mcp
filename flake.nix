{
  description = "Development environment for package-manager-mcp";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    self,
    flake-utils,
    nixpkgs,
    rust-overlay,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      overlays = [(import rust-overlay)];
      pkgs = import nixpkgs {inherit system overlays;};
    in {
      devShells.default = pkgs.mkShell {
        buildInputs = with pkgs; [alejandra (pkgs.rust-bin.stable.latest.default.override {targets = ["x86_64-unknown-linux-musl"];}) rustfmt];
      };
    });
}
