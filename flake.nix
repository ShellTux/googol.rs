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

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{
      self,
      nixpkgs,
      flake-parts,
      rust-overlay,
      pre-commit-hooks,
      treefmt-nix,
      crane,
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
          inherit (builtins)
            attrValues
            elem
            ;
          inherit (pkgs.lib) getExe filterAttrs fileset;

          additionalPackages = attrValues (
            filterAttrs (
              key: value:
              (elem key [
                "spawn-googol"
                "vm"
              ])
            ) config.packages
          );

          craneLib = crane.mkLib pkgs;

          inherit (craneLib)
            buildPackage
            cargoDoc
            cargoFmt
            cargoNextest
            ;

          src = fileset.toSource {
            root = ./.;
            fileset = fileset.unions [
              (craneLib.fileset.commonCargoSources ./.)
              # (fileset.fileFilter (file: any (ext: file.hasExt ext) ["proto"]) ./.)
              (fileset.maybeMissing ./protos)
            ];
          };

          commonArgs = {
            inherit src;
            strictDeps = true;

            nativeBuildInputs = [
              pkgs.openssl
              pkgs.openssl.dev
              pkgs.pkg-config
              pkgs.protobuf
            ];

            PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
            OPENSSL_NO_VENDOR = 1;
          };

          cargoArtifacts = craneLib.buildDepsOnly commonArgs;

          individualCrateArgs = commonArgs // {
            inherit cargoArtifacts;
            inherit (craneLib.crateNameFromCargoToml { inherit src; }) version;
            # NB: we disable tests since we'll run them all via cargo-nextest
            doCheck = false;
          };

          googol = buildPackage (commonArgs // { inherit cargoArtifacts; });

          barrel = buildPackage (
            individualCrateArgs
            // {
              pname = "barrel";
              cargoExtraArgs = "--bin=barrel";
            }
          );

          client = buildPackage (
            individualCrateArgs
            // {
              pname = "client";
              cargoExtraArgs = "--bin=client";
            }
          );

          downloader = buildPackage (
            individualCrateArgs
            // {
              pname = "downloader";
              cargoExtraArgs = "--bin=downloader";
            }
          );

          gateway = buildPackage (
            individualCrateArgs
            // {
              pname = "gateway";
              cargoExtraArgs = "--bin=gateway";
            }
          );

          web-server = buildPackage (
            individualCrateArgs
            // {
              pname = "web-server";
              cargoExtraArgs = "--bin=web-server";
            }
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
            # cargo-check.enable = true;
            check-toml.enable = true;
            # clippy.enable = true;
            # nixpkgs-fmt.enable = true;
            rustfmt.enable = true;
          };

          checks = {
            inherit googol;

            my-workspace-doc = cargoDoc (
              commonArgs
              // {
                inherit cargoArtifacts;
              }
            );

            my-workspace-fmt = cargoFmt {
              inherit src;
            };

            my-workspace-nextest = cargoNextest (
              commonArgs
              // {
                inherit cargoArtifacts;
                partitions = 1;
                partitionType = "count";
                cargoNextestPartitionsExtraArgs = "--no-tests=pass";
              }
            );
          };

          devShells =
            let
              inherit (pkgs.rust.packages.stable.rustPlatform) rustLibSrc;
              inherit (config) pre-commit;
            in
            {
              default = craneLib.devShell {
                name = "googol.nix";

                packages =
                  [
                    pkgs.openssl
                    pkgs.pkg-config
                    pkgs.cargo-deny
                    pkgs.cargo-edit
                    pkgs.cargo-nextest
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

              github-ci = craneLib.devShell {
                packages =
                  [
                    pkgs.openssl
                    pkgs.pkg-config
                    pkgs.cargo-nextest
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

          packages = {
            inherit
              googol
              client
              gateway
              barrel
              downloader
              web-server
              ;

            default = googol;
          };
        };
    };
}
