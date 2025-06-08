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
    in
    {
      packages = {
        spawn-googol = callPackage ./spawn-googol { };
        vm = callPackage ./vm { };
      };
    };
}
