// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Formatters
import struct Gemstone.Asset
import enum Gemstone.GemConfirmError
import Foundation
import GemstonePrimitives
import Localization

extension GemConfirmError: @retroactive LocalizedError {
    public var errorDescription: String? {
        switch self {
        case .Offline: Localized.Errors.networkOffline
        case .ScanMalicious: Localized.Errors.ScanTransaction.Malicious.description
        case let .ScanMemoRequired(symbol): Localized.Errors.ScanTransaction.memoRequired(symbol.boldMarkdown())
        case .FeeRatesMissing: Localized.Errors.unableEstimateNetworkFee
        case .AccountMissing, .BalanceMissing, .SenderMismatch: Localized.Errors.unknown
        case let .InsufficientBalance(asset, _, _): Localized.Transfer.insufficientBalance(Self.title(asset: asset))
        case let .InsufficientNetworkFee(asset, _, _): Localized.Transfer.insufficientNetworkFeeBalance(Self.title(asset: asset))
        case let .MinimumAccountBalanceTooLow(asset, required, _):
            Localized.Transfer.minimumAccountBalance(ValueFormatter(style: .full).string(BigInt(core: required), asset: asset.map()).boldMarkdown())
        case let .Network(msg), let .Load(msg), let .Broadcast(_, msg), let .Record(msg), let .Sign(_, msg), let .ApprovalInvalid(msg): msg
        }
    }
}

extension GemConfirmError {
    var hasInfoSheet: Bool {
        switch self {
        case .ScanMalicious, .ScanMemoRequired, .InsufficientBalance, .InsufficientNetworkFee, .MinimumAccountBalanceTooLow: true
        default: false
        }
    }

    private static func title(asset: Gemstone.Asset) -> String {
        let title = asset.name == asset.symbol ? asset.name : String(format: "%@ (%@)", asset.name, asset.symbol)
        return title.boldMarkdown()
    }
}
