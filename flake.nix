{
  description = "A Nix-flake-based Rust development environment";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    pre-commit-hooks = {
      url = "github:cachix/git-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      pre-commit-hooks,
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
            inherit system;

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

      checks = forEachSupportedSystem(
        { pkgs, system, ... }: {
          pre-commit-check = pre-commit-hooks.lib.${system}.run {
            src = ./.;
            hooks = {
              cargo-check.enable = true;
              check-toml.enable = true;
              # clippy.enable = true;
              rustfmt.enable = true;
            };
          };
        }
        );

      devShells = forEachSupportedSystem (
        { pkgs, system, ... }:
        let
          inherit (pkgs) mkShell;
          inherit (pkgs.lib) getExe;
          inherit (pkgs) rustToolchain;
          inherit (pkgs.rust.packages.stable.rustPlatform) rustLibSrc;
          inherit (self.checks.${system}) pre-commit-check;

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
              pkgs.curlie
              pkgs.entr
              pkgs.jq
              pkgs.python3
              pkgs.tokei
              pkgs.websocat
            ] ++ pre-commit-check.enabledPackages;

            env = {
              # Required by rust-analyzer
              RUST_SRC_PATH = "${rustLibSrc}";
              RUST_LOG = "debug,h2=error,tower=error,hyper_util=error,html5ever=error,selectors=error";
            };

            shellHook = ''
              ${pre-commit-check.shellHook}
              ${onefetch} --no-bots 2>/dev/null
            '';
          };

          github-ci = mkShell {
            packages = [
              rustToolchain

              pkgs.openssl
              pkgs.pkg-config
            ] ++ [
              # gRPC
              pkgs.protobuf
            ];

            env = {
              # Required by rust-analyzer
              RUST_SRC_PATH = "${rustLibSrc}";
              CARGO_TERM_COLOR = "always";
            };
          };
        }
      );

      packages = forEachSupportedSystem (
        { pkgs, ... }: let
          inherit (builtins) fromTOML readFile;
          inherit (pkgs) callPackage;
          inherit (pkgs.rustPlatform) buildRustPackage;
          inherit (pkgs.lib) getExe';

          version = (fromTOML (readFile ./Cargo.toml)).package.version;

          googol = buildRustPackage {
            name = "googol";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;

            nativeBuildInputs = [
              pkgs.openssl
              pkgs.openssl.dev
              pkgs.pkg-config
              pkgs.protobuf
            ];

            PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
            OPENSSL_NO_VENDOR = 1;
          };

          pkg-symlink = p: (let
            target = getExe' googol "${p}";
          in pkgs.runCommand "${p}" { } ''
            mkdir --parents $out/bin
            ln --symbolic ${target} $out/bin
          '');
        in
        {
          default = googol;

          barrel = pkg-symlink "barrel";
          client = pkg-symlink "client";
          downloader = pkg-symlink "downloader";
          gateway = pkg-symlink "gateway";
          web-server = pkg-symlink "web-server";

          spawn-googol = callPackage ./spawn-googol { };
        }
      );
    };
}
