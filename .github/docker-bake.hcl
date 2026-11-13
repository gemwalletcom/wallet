variable "GITHUB_REPOSITORY" {
  default = "gemwalletcom/wallet"
}

variable "GITHUB_SHA" {
  default = "local"
}

group "default" {
  targets = ["image"]
}

target "image" {
  name = image
  matrix = {
    image = ["core", "dynode"]
  }
  context = "./core"
  dockerfile = "Dockerfile"
  target = image
  platforms = ["linux/amd64"]
  labels = {
    "org.opencontainers.image.source" = "https://github.com/${GITHUB_REPOSITORY}"
  }
  tags = [
    "ghcr.io/gemwalletcom/wallet/${image}:latest",
    "ghcr.io/gemwalletcom/wallet/${image}:${GITHUB_SHA}",
  ]
  cache-from = ["type=gha,version=2,scope=${image}"]
  cache-to = ["type=gha,version=2,mode=max,scope=${image}"]
}
