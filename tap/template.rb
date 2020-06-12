class Pacaptr < Formula
    desc "Pacman syntax wrapper for many package managers"
    homepage "https://github.com/rami3l/pacaptr"
    version "{version}"
    url "{url_mac}"
    sha256 "{sha256_mac}"
    
    if OS.linux?
      url "{url_linux}"
      sha256 "{sha256_linux}"
    end

    def install
      bin.install "pacaptr"
    end
end