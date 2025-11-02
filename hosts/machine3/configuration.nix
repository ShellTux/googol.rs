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
      googol.client
      googol.web-server
    ];
  };

  system.stateVersion = "24.11";
}
