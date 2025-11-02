{ googol, ... }:
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
      googol.downloader
      googol.barrel
      googol.client
    ];
  };

  system.stateVersion = "24.11";
}
