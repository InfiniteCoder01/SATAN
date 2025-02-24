{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem
      (system: let
        target = "i686-elf";
        pkgs = (import nixpkgs { inherit system; });
        pkgs-crosssystem = (import nixpkgs {
          inherit system;
          crossSystem = {
            config = target;
          };
        });
      in
        {
          devShell = pkgs.mkShell {
            buildInputs = with pkgs; [
              qemu bochs
              libisoburn mtools
              pkgs-crosssystem.buildPackages.grub2
              pkgs-crosssystem.buildPackages.gcc
            ];
            CROSS_CC = "${pkgs-crosssystem.buildPackages.gcc.outPath}/bin/${target}-gcc";
          };
        }
      );
}
