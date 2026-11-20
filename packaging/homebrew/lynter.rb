# Template. Replace VERSION and each SHA256 from the release SHA256SUMS file,
# then publish to a tap such as klappe-pm/homebrew-tap as Formula/lynter.rb.
class Lynter < Formula
  desc "Markdown linter"
  homepage "https://lynter.dev"
  version "VERSION"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/klappe-pm/lynter/releases/download/vVERSION/lynter-vVERSION-aarch64-apple-darwin.tar.gz"
      sha256 "SHA256_AARCH64_APPLE_DARWIN"
    end
    on_intel do
      url "https://github.com/klappe-pm/lynter/releases/download/vVERSION/lynter-vVERSION-x86_64-apple-darwin.tar.gz"
      sha256 "SHA256_X86_64_APPLE_DARWIN"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/klappe-pm/lynter/releases/download/vVERSION/lynter-vVERSION-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "SHA256_AARCH64_UNKNOWN_LINUX_GNU"
    end
    on_intel do
      url "https://github.com/klappe-pm/lynter/releases/download/vVERSION/lynter-vVERSION-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "SHA256_X86_64_UNKNOWN_LINUX_GNU"
    end
  end

  def install
    bin.install "lynter"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/lynter --version")
  end
end
