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
          corepack_latest
          nodejs
          rustup
          sccache
          # python311
        ];
        shellHook = ''
          rustup toolchain install nightly-2023-10-31 -c miri rustc-codegen-cranelift-preview rust-src rust-analyzer rustfmt clippy
        '';
      };
    };
}
