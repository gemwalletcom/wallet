// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Formatters
import Foundation
import Localization
import Primitives

public enum TransferAmountCalculatorError: Equatable {
    case insufficientBalance(Asset, requirement: BalanceRequirement)
    case insufficientNetworkFee(Asset, requirement: BalanceRequirement?)
    case minimumAccountBalanceTooLow(Asset, requirement: BalanceRequirement)

    public init(_ error: TransferAmountError, asset: Asset, assetFee: Asset) {
        switch error {
        case let .insufficientBalance(assetId, requirement):
            self = .insufficientBalance(Self.asset(assetId, asset: asset, assetFee: assetFee), requirement: requirement)
        case let .insufficientNetworkFee(assetId, requirement):
            self = .insufficientNetworkFee(Self.asset(assetId, asset: asset, assetFee: assetFee), requirement: requirement)
        case let .minimumAccountBalanceTooLow(assetId, requirement):
            self = .minimumAccountBalanceTooLow(Self.asset(assetId, asset: asset, assetFee: assetFee), requirement: requirement)
        }
    }

    private static func asset(_ assetId: AssetId, asset: Asset, assetFee: Asset) -> Asset {
        asset.id == assetId ? asset : assetFee
    }
}

extension TransferAmountCalculatorError: LocalizedError {
    public var errorDescription: String? {
        switch self {
        case let .insufficientBalance(asset, _):
            Localized.Transfer.insufficientBalance(Self.title(asset: asset))
        case let .insufficientNetworkFee(asset, _):
            Localized.Transfer.insufficientNetworkFeeBalance(Self.title(asset: asset))
        case let .minimumAccountBalanceTooLow(asset, requirement):
            Localized.Transfer.minimumAccountBalance(Self.formattedValue(requirement.required, asset: asset))
        }
    }

    private static func title(asset: Asset) -> String {
        let title = asset.name == asset.symbol ? asset.name : String(format: "%@ (%@)", asset.name, asset.symbol)
        return title.boldMarkdown()
    }

    private static func formattedValue(_ value: BigInt, asset: Asset) -> String {
        ValueFormatter(style: .full).string(value, asset: asset).boldMarkdown()
    }
}
