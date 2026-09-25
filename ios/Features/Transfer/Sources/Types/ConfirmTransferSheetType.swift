// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemAcquireAsset
import struct Gemstone.GemInfoSheet
import GemstonePrimitives
import InfoSheet
import Primitives
import PrimitivesComponents

public enum ConfirmTransferSheetType: Identifiable, Sendable {
    case info(GemInfoSheet)
    case networkFeeSelector
    case paymentAsset(SelectAssetType)
    case paymentVerification(URL)
    case payloadDetails
    case fiatConnect(assetAddress: AssetAddress, wallet: Wallet, amount: Int?)
    case getAsset(Asset, acquire: GemAcquireAsset)
    case selectedAsset(SelectedAssetInput, wallet: Wallet)
    case swapDetails
    case perpetualDetails(PerpetualDetailsViewModel)
    case addressDetails(ChainAddress)

    public var id: String {
        switch self {
        case let .info(sheet): "info-\(sheet.hashValue)"
        case .networkFeeSelector: "network-fee-selector"
        case let .paymentAsset(type): "payment-asset-\(type.id)"
        case let .paymentVerification(url): "payment-verification-\(url)"
        case .payloadDetails: "payload-details"
        case .fiatConnect: "fiat-connect"
        case let .getAsset(asset, _): "get-asset-\(asset.id.identifier)"
        case let .selectedAsset(input, _): "selected-asset-\(input.id)"
        case .swapDetails: "swap-details"
        case let .perpetualDetails(model): "perpetual-details-\(model.id)"
        case let .addressDetails(chainAddress): "address-details-\(chainAddress.chain.rawValue)-\(chainAddress.address)"
        }
    }
}
