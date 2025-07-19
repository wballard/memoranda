class Memoranda < Formula
  desc "Memory-augmented note-taking system with MCP server capabilities for coding agents"
  homepage "https://github.com/wballard/memoranda"
  url "https://github.com/wballard/memoranda/archive/v0.1.0.tar.gz"
  sha256 "REPLACE_WITH_ACTUAL_SHA256"
  license "MIT"
  head "https://github.com/wballard/memoranda.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
    
    # Install shell completions
    bash_completion.install "completions/memoranda.bash" => "memoranda"
    zsh_completion.install "completions/_memoranda"
    fish_completion.install "completions/memoranda.fish"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/memoranda --version")
    
    # Test basic functionality
    system "#{bin}/memoranda", "doctor"
    
    # Test MCP server can start (with timeout)
    pid = spawn("#{bin}/memoranda", "serve")
    sleep 1
    Process.kill("TERM", pid)
    Process.wait(pid)
  end
end