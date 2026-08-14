// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import Primitives

enum PaymentAsset {
    case unsupported
    case single(AssetData)
    case choice([AssetData])
}

extension PaymentAsset {
    static func from(_ payment: PaymentRequest, assets: [AssetData]) -> PaymentAsset {
        let payable = payableAssets(for: payment, in: assets)

        guard let assetData = payable.first else { return .unsupported }
        return payable.count == 1 ? .single(assetData) : .choice(payable)
    }

    private static func payableAssets(for payment: PaymentRequest, in assets: [AssetData]) -> [AssetData] {
        guard let assetId = payment.assetId else {
            return assets.filter { $0.asset.chain.isValidAddress(payment.address) }
        }
        return assets.filter { $0.asset.id == assetId }
    }
}
