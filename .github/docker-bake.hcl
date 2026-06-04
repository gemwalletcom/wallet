variable "GITHUB_REPOSITORY" {
  default = "gemwalletcom/wallet"
}

variable "GITHUB_SHA" {
  default = "local"
}

group "default" {
  targets = ["core", "dynode"]
}

target "_common" {
  context = "./core"
  dockerfile = "Dockerfile"
  platforms = ["linux/amd64"]
  labels = {
    "org.opencontainers.image.source" = "https://github.com/${GITHUB_REPOSITORY}"
  }
}

target "core" {
  inherits = ["_common"]
  target = "core"
  tags = [
    "ghcr.io/gemwalletcom/wallet/core:latest",
    "ghcr.io/gemwalletcom/wallet/core:${GITHUB_SHA}",
  ]
  cache-from = ["type=gha,scope=core"]
  cache-to = ["type=gha,mode=max,scope=core"]
}

target "dynode" {
  inherits = ["_common"]
  target = "dynode"
  tags = [
    "ghcr.io/gemwalletcom/wallet/dynode:latest",
    "ghcr.io/gemwalletcom/wallet/dynode:${GITHUB_SHA}",
  ]
  cache-from = ["type=gha,scope=dynode"]
  cache-to = ["type=gha,mode=max,scope=dynode"]
}
