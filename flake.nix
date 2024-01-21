{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };
  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
      };
    in
    with pkgs;
    {
      devShells.${system}.default = mkShell {
        inherit nixpkgs;
        buildInputs = with pkgs.buildPackages; [
          nodePackages.pnpm
          nodejs
          rustup
          sccache
        ];
        shellHook = ''
          rustup toolchain install nightly-2024-01-20 -c miri rustc-codegen-cranelift-preview rust-src rust-analyzer rustfmt clippy
          rustup default nightly-2024-01-20
        '';
      };
    };
}
