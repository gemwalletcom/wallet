import Primitives

enum ImportWalletSceneResult: Sendable {
    case new(Wallet)
    case existing
}
