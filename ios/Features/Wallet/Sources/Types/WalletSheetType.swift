// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemInfoSheet
import enum Gemstone.GemSelectAssetType
import struct Gemstone.GemTransferData
import GemstonePrimitives
import InfoSheet
import Primitives
import PrimitivesComponents

public enum WalletSheetType: Identifiable, Equatable, Sendable {
    case selectAsset(GemSelectAssetType, chains: [Chain])
    case amount(AmountInput)
    case infoSheet(GemInfoSheet)
    case transferData(GemTransferData)
    case addAsset
    case portfolio(PortfolioType)
    case swap

    public var id: String {
        switch self {
        case let .selectAsset(type, _): "selectAsset-\(type)"
        case let .amount(input): "amount-\(input.request)-\(input.asset.id.identifier)"
        case let .infoSheet(sheet): "infoSheet-\(sheet.hashValue)"
        case let .transferData(data): "transferData-\(data.id)"
        case .addAsset: "addAsset"
        case let .portfolio(type): "portfolio-\(type.id)"
        case .swap: "swap"
        }
    }
}
