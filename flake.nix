{
  description = "An over-engineered Hello World in C";

  # Nixpkgs / NixOS version to use.
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
    }:
    let

      # to work with older version of flakes
      lastModifiedDate = self.lastModifiedDate or self.lastModified or "19700101";

      # Generate a user-friendly version number.
      version = builtins.substring 0 8 lastModifiedDate;

      # System types to support.
      supportedSystems = [
        "x86_64-linux"
        "x86_64-darwin"
        "aarch64-linux"
        "aarch64-darwin"
      ];

      # Helper function to generate an attrset '{ x86_64-linux = f "x86_64-linux"; ... }'.
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;

      # A Nixpkgs overlay.
      overlay =
        final: prev:
        let
          craneLib = crane.mkLib final;
        in
        {
          quanwutong-exporter = final.callPackage (import ./nix/package.nix) {
            inherit craneLib;
            inherit version;
          };
        };

      # Nixpkgs instantiated for supported system types.
      nixpkgsFor = forAllSystems (
        system:
        import nixpkgs {
          inherit system;
          overlays = [ overlay ];
          config = {
            allowUnfree = true;
          };
        }
      );
    in
    {

      # Provide some binary packages for selected system types.
      packages = forAllSystems (system: {
        default = nixpkgsFor.${system}.quanwutong-exporter;
      });

      nixosModules.default =
        { ... }:
        {
          imports = [ ./nix/module.nix ];
          nixpkgs.overlays = [ overlay ];
        };

      # Tests run by 'nix flake check' and by Hydra.
      checks = forAllSystems (system: {
        inherit (self.packages.${system}) default;
      });

    };
}
