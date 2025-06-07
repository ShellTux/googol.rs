{
  entr,
  simple-http-server,
  parallel-full,
  writeShellApplication,
}:
let
  parallel = parallel-full.override {
    willCite = true;
  };

  inherit (builtins) readFile;
in
writeShellApplication {
  name = "watch";

  runtimeInputs = [
    entr
    parallel
    simple-http-server
  ];

  text = readFile ./spawn-googol.sh;
}
