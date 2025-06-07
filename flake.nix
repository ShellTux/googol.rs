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

    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };

    treefmt-nix.url = "github:numtide/treefmt-nix";
  };

  outputs =
    inputs@{
      self,
      nixpkgs,
      flake-parts,
      rust-overlay,
      pre-commit-hooks,
      treefmt-nix,
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports =
        [
          ./hosts
          ./pkgs
        ]
        ++ [
          pre-commit-hooks.flakeModule
          treefmt-nix.flakeModule
        ];

      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
        "x86_64-darwin"
      ];

      perSystem =
        {
          config,
          self',
          inputs',
          pkgs,
          system,
          ...
        }:
        let
          inherit (builtins) attrValues elem;
          inherit (pkgs.lib) getExe filterAttrs;

          additionalPackages = attrValues (
            filterAttrs (
              key: value:
              (elem key [
                "spawn-googol"
                "vm"
              ])
            ) config.packages
          );

          onefetch = getExe pkgs.onefetch;
        in
        {
          _module.args.pkgs = import nixpkgs {
            inherit system;

            overlays = [ rust-overlay.overlays.default ];
          };

          treefmt = {
            programs = {
              nixfmt.enable = true;
              shellcheck.enable = true;
            };
            settings.formatter.shellcheck.excludes = [ "**/.envrc" ];
          };

          pre-commit.settings.hooks = {
            cargo-check.enable = true;
            check-toml.enable = true;
            # clippy.enable = true;
            # nixpkgs-fmt.enable = true;
            rustfmt.enable = true;
          };

          devShells =
            let
              inherit (pkgs) mkShell;
              inherit (pkgs.rust.packages.stable.rustPlatform) rustLibSrc;
              inherit (config) pre-commit;

              rustToolchain =
                let
                  rust = pkgs.rust-bin;
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
            in
            {
              default = mkShell {
                name = "googol.nix";

                packages =
                  [
                    rustToolchain

                    pkgs.openssl
                    pkgs.pkg-config
                    pkgs.cargo-deny
                    pkgs.cargo-edit
                    pkgs.cargo-watch
                    pkgs.rust-analyzer
                  ]
                  ++ [
                    # gRPC
                    pkgs.grpcui
                    pkgs.grpcurl
                    pkgs.protobuf
                  ]
                  ++ [
                    # Compiling pdf
                    pkgs.mermaid-filter
                    pkgs.pandoc
                    pkgs.pandoc-include
                    pkgs.texliveFull
                  ]
                  ++ [
                    pkgs.curlie
                    pkgs.entr
                    pkgs.jq
                    pkgs.python3
                    pkgs.tokei
                    pkgs.websocat
                  ]
                  ++ additionalPackages;

                env = {
                  # Required by rust-analyzer
                  RUST_SRC_PATH = "${rustLibSrc}";
                  RUST_LOG = "debug,h2=error,tower=error,hyper_util=error,html5ever=error,selectors=error";
                };

                shellHook = ''
                  ${pre-commit.installationScript}
                  ${onefetch} --no-bots
                '';
              };

              github-ci = mkShell {
                packages =
                  [
                    rustToolchain

                    pkgs.openssl
                    pkgs.pkg-config
                  ]
                  ++ [
                    # gRPC
                    pkgs.protobuf
                  ];

                env = {
                  # Required by rust-analyzer
                  RUST_SRC_PATH = "${rustLibSrc}";
                  CARGO_TERM_COLOR = "always";
                };
              };
            };

          packages.default =
            let
              inherit (builtins) fromTOML readFile;
              inherit (pkgs.rustPlatform) buildRustPackage;
              inherit ((fromTOML (readFile ./Cargo.toml)).package) name version;

            in
            buildRustPackage {
              inherit name version;

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
        };
    };
}
