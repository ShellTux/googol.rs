{
  self,
  inputs,
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
    nixosSystem {
      modules = [
        {
          networking.hostName = name;
          nixpkgs.hostPlatform = system;
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
