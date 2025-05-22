{
  description = "A Nix-flake-based Rust development environment";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
    }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forEachSupportedSystem =
        f:
        nixpkgs.lib.genAttrs supportedSystems (
          system:
          f {
            pkgs = import nixpkgs {
              inherit system;
              overlays = [
                rust-overlay.overlays.default
                self.overlays.default
              ];
            };
          }
        );
    in
    {
      overlays.default = final: prev: {
        rustToolchain =
          let
            rust = prev.rust-bin;
          in
          if builtins.pathExists ./rust-toolchain.toml then
            rust.fromRustupToolchainFile ./rust-toolchain.toml
          else if builtins.pathExists ./rust-toolchain then
            rust.fromRustupToolchainFile ./rust-toolchain
          else
            rust.stable.latest.default.override {
              extensions = [
                "rust-src"
                "rustfmt"
              ];
            };
      };

      devShells = forEachSupportedSystem (
        { pkgs }:
        let
          inherit (pkgs) mkShell;
          inherit (pkgs.lib) getExe;
          inherit (pkgs) rustToolchain;
          inherit (pkgs.rust.packages.stable.rustPlatform) rustLibSrc;

          onefetch = getExe pkgs.onefetch;
        in
        {
          default = mkShell {
            packages = [
              rustToolchain

              pkgs.openssl
              pkgs.pkg-config
              pkgs.cargo-deny
              pkgs.cargo-edit
              pkgs.cargo-watch
              pkgs.rust-analyzer
            ] ++ [
              # gRPC
              pkgs.grpcui
              pkgs.grpcurl
              pkgs.protobuf
            ] ++ [
              # Compiling pdf
              pkgs.mermaid-filter
              pkgs.pandoc
              pkgs.pandoc-include
              pkgs.texliveFull
            ] ++ [
              pkgs.entr
              pkgs.jq
              pkgs.simple-http-server
              pkgs.tokei
              pkgs.websocat
            ];

            env = {
              # Required by rust-analyzer
              RUST_SRC_PATH = "${rustLibSrc}";
              RUST_LOG = "debug,h2=error,tower=error,hyper_util=error";
            };

            shellHook = ''
              ${onefetch} --no-bots 2>/dev/null
            '';
          };
        }
      );
    };
}
