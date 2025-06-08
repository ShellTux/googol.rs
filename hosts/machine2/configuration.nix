{ pkgs, googol, ... }:
{
  boot.loader.grub.devices = [ "nodev" ];

  users.users.googol = {
    isNormalUser = true;
    extraGroups = [ "wheel" ]; # Enable ‘sudo’ for the user.
    initialPassword = "googol";
  };

  environment = {
    variables = {
      TERM = "screen-256color";
    };

    systemPackages = [
      pkgs.bat
      pkgs.btop
      pkgs.cowsay
      pkgs.curl
      pkgs.htop
      pkgs.lolcat
      pkgs.tldr
      pkgs.tmux
      pkgs.vim
    ] ++ [ googol ];
  };

  system.stateVersion = "24.11";
}
