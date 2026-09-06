// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Formatters
import Foundation
import struct Gemstone.Asset
import enum Gemstone.GemConfirmError
import GemstonePrimitives
import Localization
import Primitives

extension GemConfirmError: @retroactive LocalizedError {
    public var errorDescription: String? {
        switch self {
        case .Offline: Localized.Errors.networkOffline
        case .ScanMalicious: Localized.Errors.ScanTransaction.Malicious.description
        case let .ScanMemoRequired(symbol): Localized.Errors.ScanTransaction.memoRequired(symbol.boldMarkdown())
        case .FeeRatesMissing: Localized.Errors.unableEstimateNetworkFee
        case .Cancelled: Localized.Errors.cancelled
        case .AccountMissing, .BalanceMissing, .SenderMismatch: Localized.Errors.unknown
        case let .InsufficientBalance(asset, requirement):
            Localized.Info.balanceRequiredDescription(
                Self.amount(requirement.required, asset: asset).boldMarkdown(),
                Self.amount(requirement.available, asset: asset),
                Self.amount(requirement.shortfall, asset: asset),
            )
        case let .InsufficientNetworkFee(asset, requirement):
            if let requirement {
                Localized.Info.InsufficientNetworkFeeBalance.description(
                    Self.amount(requirement.required, asset: asset).boldMarkdown(),
                    asset.map().chain.networkName.boldMarkdown(),
                    Self.amount(requirement.available, asset: asset),
                    Self.amount(requirement.shortfall, asset: asset),
                )
            } else {
                Localized.Transfer.insufficientNetworkFeeBalance(Self.title(asset: asset))
            }
        case let .MinimumAccountBalanceTooLow(asset, requirement):
            Localized.Transfer.minimumAccountBalance(ValueFormatter(style: .full).string(requirement.required, asset: asset.map()).boldMarkdown())
        case let .BelowSwapMinimum(asset, _, providerName, requirement):
            Localized.Info.swapMinimumAmountDescription(
                providerName.boldMarkdown(),
                Self.amount(requirement.required, asset: asset).boldMarkdown(),
                Self.amount(requirement.available, asset: asset),
                Self.amount(requirement.shortfall, asset: asset),
            )
        case .Sign(.dustThreshold, _, _): Localized.Errors.dustThresholdShort
        case .Sign(.insufficientFunds, _, _): Localized.Info.InsufficientBalance.title
        case let .Network(msg), let .Load(msg), let .Broadcast(_, msg), let .Record(msg), let .Sign(_, _, msg), let .ApprovalInvalid(msg): msg
        }
    }
}

extension GemConfirmError {
    var hasInfoSheet: Bool {
        switch self {
        case .ScanMalicious, .ScanMemoRequired, .InsufficientBalance, .InsufficientNetworkFee, .MinimumAccountBalanceTooLow, .BelowSwapMinimum, .Sign(.dustThreshold, _, _): true
        default: false
        }
    }

    private static func amount(_ value: BigInt, asset: Gemstone.Asset) -> String {
        ValueFormatter(style: .full).string(value, asset: asset.map())
    }

    private static func title(asset: Gemstone.Asset) -> String {
        let title = asset.name == asset.symbol ? asset.name : String(format: "%@ (%@)", asset.name, asset.symbol)
        return title.boldMarkdown()
    }
}
