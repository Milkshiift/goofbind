{ pkgs, lib, config, inputs, ... }:

{
  packages = with pkgs; [
    cmake
    libclang
    pkg-config

    xorg.libX11
    xorg.libXi
    xorg.libXtst
    xorg.libxcb
    libxkbcommon
    xorg.libxkbfile

    wayland

    ninja
    llvmPackages_latest.llvm
    cargo-xwin
    cargo-zigbuild
  ];
  env.LIBCLANG_PATH="${pkgs.libclang.lib}/lib";
  env.CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS=
    "-Clink-arg=-L${pkgs.pkgsCross.aarch64-multiplatform.libxkbcommon}/lib " +
    "-Clink-arg=-L${pkgs.pkgsCross.aarch64-multiplatform.xorg.libxcb}/lib " +
    "-Clink-arg=-L${pkgs.pkgsCross.aarch64-multiplatform.xorg.libXtst}/lib " +
    "-Clink-arg=-L${pkgs.pkgsCross.aarch64-multiplatform.xorg.libX11}/lib " +
    "-Clink-arg=-L${pkgs.pkgsCross.aarch64-multiplatform.wayland}/lib";
  env.PKG_CONFIG_PATH_aarch64_unknown_linux_gnu=
    "${pkgs.pkgsCross.aarch64-multiplatform.xorg.libX11.dev.outPath}/lib/pkgconfig:" +
    "${pkgs.xorg.xorgproto}/share/pkgconfig:" +
    "${pkgs.pkgsCross.aarch64-multiplatform.wayland.dev.outPath}/lib/pkgconfig";
  enterShell = ''
    export BINDGEN_EXTRA_CLANG_ARGS="$NIX_CFLAGS_COMPILE \
      $(< ${pkgs.clang}/nix-support/libc-cflags) \
      $(< ${pkgs.clang}/nix-support/cc-cflags)"
  '';
  languages = {
    rust = {
      enable = true;
      channel = "stable";
      mold.enable = false;
      targets = [
        "aarch64-unknown-linux-gnu"
        "x86_64-unknown-linux-gnu"
        "aarch64-pc-windows-msvc"
        # "x86_64-pc-windows-msvc"
        # "aarch64-apple-darwin"
        # "x86_64-apple-darwin"
      ];
    };
    javascript = {
      enable = true;
      pnpm.enable = true;
    };
  };
}
