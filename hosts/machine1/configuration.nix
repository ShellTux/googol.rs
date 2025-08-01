{ pkgs, googol, ... }:
{
  users.users.googol = {
    isNormalUser = true;
    extraGroups = [ "wheel" ]; # Enable ‘sudo’ for the user.
    initialPassword = "googol";
  };

  environment = {
    variables = {
      TERM = "screen-256color";
    };

    systemPackages =
      [
        pkgs.bat
        pkgs.btop
        pkgs.cowsay
        pkgs.curl
        pkgs.htop
        pkgs.lolcat
        pkgs.tldr
        pkgs.tmux
        pkgs.vim
      ]
      ++ [
        googol.downloader
        googol.barrel
        googol.gateway
      ];
  };

  system.stateVersion = "24.11";
}
