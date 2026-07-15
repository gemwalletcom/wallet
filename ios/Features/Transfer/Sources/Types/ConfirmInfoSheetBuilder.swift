// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import InfoSheet
import Primitives
import PrimitivesComponents
import Validators

enum ConfirmInfoSheetBuilder {
    static func build(
        for error: ConfirmTransferError,
        asset: Asset,
        feePrice: Price?,
        currency: String,
        onGetNetworkFeeAsset: @escaping @MainActor @Sendable () -> Void,
    ) -> InfoSheetType? {
        switch error {
        case let .amount(error):
            amountSheet(for: error, feePrice: feePrice, currency: currency, onGetNetworkFeeAsset: onGetNetworkFeeAsset)
        case let .scan(error):
            scanSheet(for: error)
        case .chain(.dustThreshold):
            .dustThreshold(asset.chain, image: image(for: asset))
        case .chain, .other:
            nil
        }
    }

    private static func amountSheet(
        for error: TransferAmountCalculatorError,
        feePrice: Price?,
        currency: String,
        onGetNetworkFeeAsset: @escaping @MainActor @Sendable () -> Void,
    ) -> InfoSheetType {
        switch error {
        case let .insufficientBalance(asset):
            .insufficientBalance(asset, image: image(for: asset))
        case let .insufficientNetworkFee(asset, required):
            .insufficientNetworkFee(asset, image: image(for: asset), required: required, price: feePrice, currency: currency, action: onGetNetworkFeeAsset)
        case let .minimumAccountBalanceTooLow(asset, required):
            .accountMinimalBalance(asset, required: required)
        }
    }

    private static func scanSheet(for error: ScanTransactionError) -> InfoSheetType {
        switch error {
        case .malicious:
            .maliciousTransaction
        case let .memoRequired(symbol):
            .memoRequired(symbol: symbol)
        }
    }

    private static func image(for asset: Asset) -> AssetImage {
        AssetViewModel(asset: asset).assetImage
    }
}
