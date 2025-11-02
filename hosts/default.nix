{
  self,
  inputs,
  pkgs,
  ...
}:
let
  inherit (inputs.nixpkgs.lib) nixosSystem;

  mkHost =
    {
      name,
      system,
      extraModules ? [
      ],
      extraSpecialArgs ? {
        googol = {
          inherit (self.packages."${system}")
            client
            gateway
            downloader
            barrel
            web-server
            ;
        };
      },
    }:
    let
      pkgs = inputs.nixpkgs.legacyPackages.${system};
    in
    nixosSystem {
      modules = [
        {
          networking.hostName = name;
          nixpkgs.hostPlatform = system;
          environment = {
            systemPackages = [
              pkgs.bat
              pkgs.btop
              pkgs.cowsay
              pkgs.curl
              pkgs.htop
              pkgs.lolcat
              pkgs.net-tools
              pkgs.tldr
              pkgs.tmux
              pkgs.vim
            ];
            etc = {
              "googol/googol.toml".source = ../example.googol.toml;
              "googol/barrel.toml".source = ../examples/config/barrel.toml;
              "googol/client.toml".source = ../examples/config/client.toml;
              "googol/downloader.toml".source = ../examples/config/downloader.toml;
              "googol/gateway.toml".source = ../examples/config/gateway.toml;
              "googol/web-server.toml".source = ../examples/config/web-server.toml;
            };
            loginShellInit = ''
              printf '\n\033[31m%s\033[0m\n' 'Programs Available:'
              echo $PATH | tr ':' '\n' | while read dir
              do
                      find -L "$dir" -type f -executable -printf '%f\n' 2>/dev/null
              done | grep --extended-regexp 'barrel|client|downloader|gateway|googol|web-server' | sort --unique
            '';
            sessionVariables.RUST_LOG = "error,googol=debug,downloader=debug,client=debug,barrel=debug,gateway=debug,web-server=debug";
          };
        }
        ./${name}
      ] ++ extraModules;

      specialArgs = {
        inherit inputs self;
      } // extraSpecialArgs;
    };
in
{
  flake.nixosConfigurations = {

    machine1 = mkHost {
      name = "machine1";
      system = "x86_64-linux";
    };

    machine2 = mkHost {
      name = "machine2";
      system = "x86_64-linux";
    };

    machine3 = mkHost {
      name = "machine3";
      system = "x86_64-linux";
    };

  };
}
