import Primitives

public extension PerpetualPosition {
    var baseAsset: Asset {
        Chain.hyperCore.defaultAsset(type: .perpetual)
    }
}
