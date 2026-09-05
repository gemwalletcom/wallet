// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemAssetAction
import struct Gemstone.GemPaymentRecipient
import Primitives

public enum SelectAssetType: Identifiable, Hashable, Sendable {
    case send(GemPaymentRecipient?)
    case receive(ReceiveAssetType)
    case buy
    case swap(SelectAssetSwapType)
    case manage
    case priceAlert
    case deposit
    case withdraw

    public var id: String {
        switch self {
        case .send: "send"
        case let .receive(type): "receive_\(type.id)"
        case .buy: "buy"
        case let .swap(type): "swap_\(type.id)"
        case .manage: "manage"
        case .priceAlert: "priceAlert"
        case .deposit: "perps"
        case .withdraw: "perps_withdrawal"
        }
    }
}

public extension SelectAssetType {
    var action: GemAssetAction? {
        switch self {
        case .send: .send
        case .receive: .receive
        case .buy: .buy
        case .swap(.pay): .swapPay
        case .swap(.receive): .swapReceive
        case .manage, .priceAlert, .deposit, .withdraw: .none
        }
    }
}

public enum SelectAssetSwapType: Identifiable, Hashable, Sendable {
    case pay
    case receive(chains: [Chain], assetIds: [AssetId])

    public var id: String {
        switch self {
        case .pay: "pay"
        case let .receive(chains, assetIds): "receive_\(chains)_\(assetIds)"
        }
    }
}
