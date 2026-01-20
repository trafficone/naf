{
  stdenv,
  lib,
  zstd,
  pkg-config
}:

stdenv.mkDerivation {
  pname = "naf";
  version = "1.3.0";
  src = ./.;
  nativeBuildInputs = [pkg-config];
  buildInputs = [zstd];
  makeFlags = [
    "prefix=$(out)"
  ];
  postPatch = ''
    sed -i '/zstd\/lib/d' Makefile
    substituteInPlace **/Makefile \
      --replace "../zstd/lib/libzstd.a" "-lzstd" \
      --replace "-I../zstd/lib" ""
  '';
  preBuild = ''
    # This provides the correct -I and -L flags for the Nix store versions
    export NIX_CFLAGS_COMPILE="$NIX_CFLAGS_COMPILE $(pkg-config --cflags libzstd)"
    export NIX_LDFLAGS="$NIX_LDFLAGS $(pkg-config --libs libzstd)"
  '';
  installPhanse = ''
    runHook preInstall
    mkdir -p $out/bin
    cp ennaf/ennaf $out/bin/
    cp unnaf/unnaf $out/bin/
    runHook postInstall
  '';
  meta = with lib; {
    description = "Nucleotide Archive Format compression and decompression software";
    platforms = platforms.linux;
  };
}
