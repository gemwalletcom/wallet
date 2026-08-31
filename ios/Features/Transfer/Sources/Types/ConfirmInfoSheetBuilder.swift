// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import GemstonePrimitives
import InfoSheet
import enum Gemstone.GemConfirmError
import Primitives
import PrimitivesComponents
import Validators

enum ConfirmInfoSheetBuilder {
    static func build(
        for error: ConfirmTransferError,
        asset: Asset,
        feePrice: Price?,
        currency: String,
        onGetAsset: @escaping @MainActor @Sendable (Asset, Int?) -> Void,
    ) -> InfoSheetType? {
        switch error {
        case let .amount(error):
            amountSheet(for: error, feePrice: feePrice, currency: currency, onGetAsset: onGetAsset)
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
        onGetAsset: @escaping @MainActor @Sendable (Asset, Int?) -> Void,
    ) -> InfoSheetType {
        switch error {
        case let .insufficientBalance(asset, requirement):
            .balanceRequired(asset, image: image(for: asset), requirement: requirement, action: { onGetAsset(asset, nil) })
        case let .insufficientNetworkFee(asset, requirement):
            .insufficientNetworkFee(asset, image: image(for: asset), requirement: requirement, price: feePrice, currency: currency, action: {
                onGetAsset(asset, FiatConfig.insufficientNetworkFeeBuyAmount)
            })
        case let .minimumAccountBalanceTooLow(asset, requirement):
            .accountMinimalBalance(asset, required: requirement.required)
        }
    }

    private static func scanSheet(for error: GemConfirmError) -> InfoSheetType? {
        switch error {
        case .ScanMalicious: .maliciousTransaction
        case let .ScanMemoRequired(symbol): .memoRequired(symbol: symbol)
        default: nil
        }
    }

    private static func image(for asset: Asset) -> AssetImage {
        AssetViewModel(asset: asset).assetImage
    }
}
