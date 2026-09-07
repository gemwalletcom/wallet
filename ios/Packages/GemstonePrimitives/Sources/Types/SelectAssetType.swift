// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemAssetAction
import struct Gemstone.GemPaymentRecipient
import struct Gemstone.GemSelectAssetFlow
import enum Gemstone.GemSelectAssetType
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
    var flowType: GemSelectAssetType {
        switch self {
        case .send: .send
        case .receive(.asset): .receive
        case .receive(.collection): .receiveCollection
        case .buy: .buy
        case .swap(.pay): .swapPay
        case .swap(.receive): .swapReceive
        case .manage: .manage
        case .priceAlert: .priceAlert
        case .deposit: .deposit
        case .withdraw: .withdraw
        }
    }

    var flow: GemSelectAssetFlow {
        flowType.flow()
    }

    var action: GemAssetAction? {
        flow.action
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
