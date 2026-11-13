// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import Transfer
import Wallet

enum NavigationSheetType {
    case assetInput(SelectedAssetInput)
    case payment(PaymentDestination)
    case priceAlert(Asset)
    case addressDetails(ChainAddress)
    case addContact(AddContactType)
    case support
    case wallets
    case wallet(WalletSheetType)

    var id: String {
        switch self {
        case let .assetInput(input): "assetInput-\(input.id)"
        case let .payment(destination): "payment-\(destination.id)"
        case let .priceAlert(asset): "priceAlert-\(asset.id.identifier)"
        case let .addressDetails(chainAddress): "addressDetails-\(chainAddress.chain.rawValue)-\(chainAddress.address)"
        case let .addContact(type): "addContact-\(type.id)"
        case .support: "support"
        case .wallets: "wallets"
        case let .wallet(sheet): "wallet-\(sheet.id)"
        }
    }

    var assetInput: SelectedAssetInput? {
        switch self {
        case let .assetInput(input): input
        default: nil
        }
    }

    var walletSheet: WalletSheetType? {
        switch self {
        case let .wallet(sheet): sheet
        default: nil
        }
    }
}

struct PresentedSheet: Identifiable {
    let id = UUID()
    let type: NavigationSheetType
}
