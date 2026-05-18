class Engram < Formula
  desc "Agent Issue Management System with MCP and CLI"
  homepage "https://github.com/gorillaKim/engram"
  version "0.1.0"

  if OS.mac?
    if Hardware::CPU.arm?
      url "https://github.com/gorillaKim/engram/releases/download/v0.1.0/engram-0.1.0-aarch64-apple-darwin.tar.gz"
      sha256 "adf2949976147ba716ba16b32020e928223d2b0da36821cbe8b9a0034f827c21"
    else
      url "https://github.com/gorillaKim/engram/releases/download/v0.1.0/engram-0.1.0-x86_64-apple-darwin.tar.gz"
      sha256 "b82f8653bc64ed5fac7973cb02e7c8edfc3a302fdcc2c06be4adf84364ab025f"
    end
  end

  def install
    bin.install "engram"
  end

  test do
    system "#{bin}/engram", "--version"
  end
end
