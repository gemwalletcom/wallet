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

    static func with(asset: Asset) -> AssetData {
        with(
            asset: asset,
            account: Account(chain: asset.chain, address: "", derivationPath: "", extendedPublicKey: nil),
        )
    }

    static func with(asset: Asset, account: Account) -> AssetData {
        AssetData(
            asset: asset,
            balance: .zero,
            account: account,
            price: nil,
            priceAlerts: [],
            metadata: AssetMetaData(
                isEnabled: false,
                isBalanceEnabled: false,
                isBuyEnabled: false,
                isSellEnabled: false,
                isSwapEnabled: false,
                isStakeEnabled: false,
                isEarnEnabled: false,
                isPinned: false,
                isActive: true,
                stakingApr: nil,
                earnApr: nil,
                rankScore: 0,
            ),
            associations: [],
        )
    }
}
