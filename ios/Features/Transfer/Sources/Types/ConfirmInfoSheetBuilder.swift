// Copyright (c). Gem Wallet. All rights reserved.

import Blockchain
import Components
import Foundation
import InfoSheet
import Primitives
import PrimitivesComponents
import Validators

enum ConfirmInfoSheetBuilder {
    static func build(
        for error: Error,
        asset: Asset,
        feePrice: Price?,
        currency: String,
        onGetNetworkFeeAsset: @escaping @MainActor @Sendable () -> Void,
    ) -> InfoSheetType? {
        switch error {
        case let TransferAmountCalculatorError.insufficientBalance(asset):
            .insufficientBalance(asset, image: image(for: asset))
        case let TransferAmountCalculatorError.insufficientNetworkFee(asset, required):
            .insufficientNetworkFee(asset, image: image(for: asset), required: required, price: feePrice, currency: currency, action: onGetNetworkFeeAsset)
        case let TransferAmountCalculatorError.minimumAccountBalanceTooLow(asset, required):
            .accountMinimalBalance(asset, required: required)
        case ScanTransactionError.malicious:
            .maliciousTransaction
        case let ScanTransactionError.memoRequired(symbol):
            .memoRequired(symbol: symbol)
        default:
            dustSheet(for: error, asset: asset)
        }
    }

    private static func dustSheet(for error: Error, asset: Asset) -> InfoSheetType? {
        guard case .dustThreshold? = ChainCoreError.fromError(error) else { return nil }
        return .dustThreshold(asset.chain, image: image(for: asset))
    }

    private static func image(for asset: Asset) -> AssetImage {
        AssetViewModel(asset: asset).assetImage
    }
}
