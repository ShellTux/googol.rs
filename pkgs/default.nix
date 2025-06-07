{
  perSystem =
    {
      inputs',
      self',
      pkgs,
      system,
      ...
    }:
    let
      inherit (pkgs) callPackage;
      inherit (pkgs.lib) getExe';

      googol = self'.packages.default;

      pkg-symlink =
        p:
        (
          let
            target = getExe' googol "${p}";
          in
          pkgs.runCommand "${p}" { } ''
            mkdir --parents $out/bin
            ln --symbolic ${target} $out/bin
          ''
        );
    in
    {
      packages = {
        inherit googol;

        barrel = pkg-symlink "barrel";
        client = pkg-symlink "client";
        downloader = pkg-symlink "downloader";
        gateway = pkg-symlink "gateway";
        web-server = pkg-symlink "web-server";

        spawn-googol = callPackage ./spawn-googol { };
        vm = callPackage ./vm { };

      };
    };
}
