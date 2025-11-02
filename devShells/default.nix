{
  self',
  pkgs,
  craneLib,
  ...
}:
let
  inherit (self'.packages)
    googol
    client
    gateway
    barrel
    downloader
    web-server
    ;

  machineShell =
    {
      name,
      googolPkgs ? [ googol ],
    }:
    craneLib.devShell {
      inherit name;

      packages =
        [
          pkgs.openssl
        ]
        ++ [
          # gRPC
          pkgs.grpcui
          pkgs.grpcurl
        ]
        ++ [
          pkgs.asciinema
          pkgs.asciinema-agg
          pkgs.curlie
          pkgs.entr
          pkgs.gnuplot
          pkgs.jq
          pkgs.net-tools
          pkgs.python3
          pkgs.tokei
          pkgs.websocat
        ]
        ++ [
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
        ]
        ++ googolPkgs;

      env.RUST_LOG = "error,googol=debug,downloader=debug,client=debug,barrel=debug,gateway=debug,web-server=debug";

      shellHook = ''
        printf '\n\033[31m%s\033[0m\n' 'Programs Available:'
        for dir in ${toString googolPkgs}
        do
          find -L "$dir" -type f -executable -printf '%f\n' 2>/dev/null
        done | sort --unique
      '';
    };
in
{
  devShells = {
    machine1 = machineShell {
      name = "Machine 1 DevShell";
      googolPkgs = [
        downloader
        barrel
        gateway
      ];
    };

    machine2 = machineShell {
      name = "Machine 2 DevShell";
      googolPkgs = [
        downloader
        barrel
        client
      ];
    };

    machine3 = machineShell {
      name = "Machine 3 DevShell";
      googolPkgs = [
        client
        web-server
      ];
    };
  };

}
