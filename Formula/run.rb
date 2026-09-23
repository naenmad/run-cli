class Run < Formula
  desc "Productivity CLI utility for macOS with dual-mode interaction"
  homepage "https://github.com/naenmad/run-cli"
  version "0.3.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/naenmad/run-cli/releases/download/v0.3.0/run-macos-aarch64.tar.gz"
      sha256 "2f3b425d3fcda9d0186ab96a15d6718898193cee2f72bdc48fba6b8be6d9ef28"
    end
  end

  def install
    bin.install "run"
  end

  def caveats
    <<~EOS
      To enable in-place directory switching and shell tab completion, add to ~/.zshrc:
        eval "$(run init)"
    EOS
  end

  test do
    assert_match "run #{version}", shell_output("#{bin}/run --version")
  end
end
