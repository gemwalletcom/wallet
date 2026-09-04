import Foundation

extension AssetData: Identifiable {
    public var id: String {
        asset.id.identifier
    }
}

public extension AssetData {
    var assetAddress: AssetAddress {
        AssetAddress(asset: asset, address: account.address)
    }
}
