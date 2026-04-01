// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import InfoSheet
import Primitives

public enum WalletSheetType: Identifiable, Equatable, Sendable {
    case wallets
    case selectAsset(SelectAssetType)
    case infoSheet(InfoSheetType)
    case transferData(TransferData)
    case perpetualRecipientData(PerpetualRecipientData)
    case setPriceAlert(Asset)
    case addAsset
    case portfolio

    public var id: String {
        switch self {
        case .wallets: "wallets"
        case let .selectAsset(type): "selectAsset-\(type.id)"
        case let .infoSheet(type): "infoSheet-\(type.id)"
        case let .transferData(data): "transferData-\(data.id)"
        case .perpetualRecipientData: "perpetualRecipientData"
        case let .setPriceAlert(asset): "setPriceAlert-\(asset.id.identifier)"
        case .addAsset: "addAsset"
        case .portfolio: "portfolio"
        }
    }
}
