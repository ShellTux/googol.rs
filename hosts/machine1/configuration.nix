{ googol, ... }:
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

    systemPackages = [
      googol.downloader
      googol.barrel
      googol.gateway
    ];
  };

  system.stateVersion = "24.11";
}
