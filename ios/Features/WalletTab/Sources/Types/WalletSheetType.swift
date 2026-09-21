// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemPerpetualPositionAction
import struct Gemstone.GemTransferData
import GemstonePrimitives
import InfoSheet
import Primitives
import PrimitivesComponents

public enum WalletSheetType: Identifiable, Equatable, Sendable {
    case selectAsset(SelectAssetType, chains: [Chain])
    case infoSheet(InfoSheetType)
    case transferData(GemTransferData)
    case perpetualPosition(GemPerpetualPositionAction)
    case addAsset
    case portfolio(PortfolioType)
    case addContact(AddContactType)
    case addressDetails(ChainAddress)
    case swap

    public var id: String {
        switch self {
        case let .selectAsset(type, _): "selectAsset-\(type.id)"
        case let .infoSheet(type): "infoSheet-\(type.id)"
        case let .transferData(data): "transferData-\(data.id)"
        case .perpetualPosition: "perpetualPosition"
        case .addAsset: "addAsset"
        case let .portfolio(type): "portfolio-\(type.id)"
        case let .addContact(type): "addContact-\(type.id)"
        case let .addressDetails(chainAddress): "addressDetails-\(chainAddress.chain.rawValue)-\(chainAddress.address)"
        case .swap: "swap"
        }
    }
}
