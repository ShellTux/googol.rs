{
  coreutils,
  nixos-rebuild,
  writeShellApplication,
}:
let
  inherit (builtins) readFile;
in
writeShellApplication {
  name = "vm";

  runtimeInputs = [
    coreutils
    nixos-rebuild
  ];

  text = readFile ./vm.sh;
}
