// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import InfoSheet
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import struct Gemstone.GemTransferData

public enum WalletSheetType: Identifiable, Equatable, Sendable {
    case selectAsset(SelectAssetType, chains: [Chain])
    case infoSheet(InfoSheetType)
    case transferData(GemTransferData)
    case perpetualRecipientData(PerpetualRecipientData)
    case addAsset
    case portfolio(PortfolioType)
    case addContact(AddContactType)

    public var id: String {
        switch self {
        case let .selectAsset(type, _): "selectAsset-\(type.id)"
        case let .infoSheet(type): "infoSheet-\(type.id)"
        case let .transferData(data): "transferData-\(data.id)"
        case .perpetualRecipientData: "perpetualRecipientData"
        case .addAsset: "addAsset"
        case let .portfolio(type): "portfolio-\(type.id)"
        case let .addContact(type): "addContact-\(type.id)"
        }
    }
}
