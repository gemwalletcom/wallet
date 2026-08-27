// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemConfirmError
import Foundation
import Localization

extension GemConfirmError: @retroactive LocalizedError {
    public var errorDescription: String? {
        switch self {
        case .Offline: Localized.Errors.networkOffline
        case .ScanMalicious: Localized.Errors.ScanTransaction.Malicious.description
        case let .ScanMemoRequired(symbol): Localized.Errors.ScanTransaction.memoRequired(symbol.boldMarkdown())
        case .FeeRatesMissing: Localized.Errors.unableEstimateNetworkFee
        case .AccountMissing: Localized.Errors.unknown
        case let .Network(msg), let .Load(msg), let .Broadcast(_, msg), let .Record(msg), let .Sign(_, msg), let .ApprovalInvalid(msg): msg
        }
    }
}
