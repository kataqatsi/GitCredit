# Homebrew formula for GitCredit CLI.
# Update version, URLs, and sha256 values when cutting a release.
#
# Typical tap layout:
#   github.com/kataqatsi/homebrew-gitcredit/Formula/gitcredit.rb
#
# Install (once published):
#   brew install kataqatsi/gitcredit/gitcredit

class Gitcredit < Formula
  desc "Git contribution graph outside of GitHub"
  homepage "https://gitcredit.dev"
  license "MIT"
  version "0.1.0"

  on_macos do
    on_intel do
      url "https://github.com/kataqatsi/GitCredit/releases/download/v0.1.0/gitcredit-x86_64-apple-darwin.tar.gz"
      sha256 "REPLACE_AFTER_RELEASE"
    end
    on_arm do
      url "https://github.com/kataqatsi/GitCredit/releases/download/v0.1.0/gitcredit-aarch64-apple-darwin.tar.gz"
      sha256 "REPLACE_AFTER_RELEASE"
    end
  end

  def install
    bin.install "gitcredit"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/gitcredit --version")
  end
end
