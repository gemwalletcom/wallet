// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import InfoSheet
import GemstonePrimitives
import Primitives
import PrimitivesComponents

public enum ConfirmTransferSheetType: Identifiable, Sendable {
    case info(InfoSheetType)
    case networkFeeSelector
    case payloadDetails
    case url(URL)
    case fiatConnect(assetAddress: AssetAddress, wallet: Wallet, amount: Int?)
    case getAsset(Asset, buyAmount: Int?)
    case selectedAsset(SelectedAssetInput, wallet: Wallet)
    case swapDetails
    case perpetualDetails(PerpetualDetailsViewModel)
    case addContact(AddContactType)

    public var id: String {
        switch self {
        case let .info(type): "info-\(type.id)"
        case let .url(url): "url-\(url)"
        case .networkFeeSelector: "network-fee-selector"
        case .payloadDetails: "payload-details"
        case .fiatConnect: "fiat-connect"
        case let .getAsset(asset, _): "get-asset-\(asset.id.identifier)"
        case let .selectedAsset(input, _): "selected-asset-\(input.id)"
        case .swapDetails: "swap-details"
        case let .perpetualDetails(model): "perpetual-details-\(model.id)"
        case let .addContact(type): "add-contact-\(type.id)"
        }
    }
}
