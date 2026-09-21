// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import func Gemstone.emptyState
import struct Gemstone.GemEmptyState
import enum Gemstone.GemEmptyStateAction
import struct Gemstone.GemEmptyStateInput
import enum Gemstone.GemEmptyStateKind
import Localization
import Primitives
import Style
import SwiftUI

public struct EmptyContentTypeViewModel: EmptyContentViewable {
    public let type: EmptyContentType
    private let state: GemEmptyState

    public init(type: EmptyContentType) {
        self.type = type
        state = emptyState(
            input: GemEmptyStateInput(
                kind: type.kind,
                isViewOnly: type.isViewOnly,
                offeredActions: Array(type.actions.keys),
            ),
        )
    }

    public var title: String {
        state.title.text(symbol: type.symbol)
    }

    public var description: String? {
        state.description?.text(symbol: type.symbol)
    }

    public var image: Image? {
        state.image.image
    }

    public var buttons: [EmptyAction] {
        let actions = type.actions
        return state.actions.compactMap { action in
            actions[action].map { EmptyAction(title: action.title, action: $0) }
        }
    }
}

private extension EmptyContentType {
    var kind: GemEmptyStateKind {
        switch self {
        case .nfts: .nfts
        case .priceAlerts: .priceAlerts
        case .asset: .asset
        case .activity: .activity
        case .stake: .stake
        case .earn: .earn
        case .validators: .validators
        case .walletConnect: .walletConnect
        case .notifications: .notifications
        case .recents: .recents
        case .contacts: .contacts
        case .networkAssets: .networkAssets
        case let .search(searchType, _):
            switch searchType {
            case .assets: .searchAssets
            case .networks: .searchNetworks
            case .activity: .searchActivity
            case .perpetuals: .searchPerpetuals
            }
        }
    }

    var isViewOnly: Bool {
        switch self {
        case let .asset(_, _, _, isViewOnly): isViewOnly
        case let .activity(_, _, isViewOnly): isViewOnly
        case .nfts, .priceAlerts, .stake, .earn, .validators, .walletConnect, .notifications, .recents, .contacts, .networkAssets, .search: false
        }
    }

    var symbol: String {
        switch self {
        case let .asset(symbol, _, _, _): symbol
        case let .stake(symbol): symbol
        case let .earn(symbol): symbol
        case .nfts, .priceAlerts, .activity, .validators, .walletConnect, .notifications, .recents, .contacts, .networkAssets, .search: .empty
        }
    }

    var actions: [GemEmptyStateAction: () -> Void] {
        switch self {
        case let .nfts(receive):
            [GemEmptyStateAction.receive: receive].compactMapValues { $0 }
        case let .asset(_, buy, swap, _):
            [GemEmptyStateAction.buy: buy, .swap: swap].compactMapValues { $0 }
        case let .activity(receive, buy, _):
            [GemEmptyStateAction.buy: buy, .receive: receive].compactMapValues { $0 }
        case let .networkAssets(manage):
            [GemEmptyStateAction.manageTokenList: manage].compactMapValues { $0 }
        case let .search(searchType, action):
            switch searchType {
            case .assets: [GemEmptyStateAction.addCustomToken: action].compactMapValues { $0 }
            case .activity: [GemEmptyStateAction.clearFilters: action].compactMapValues { $0 }
            case .networks, .perpetuals: [:]
            }
        case .priceAlerts, .stake, .earn, .validators, .walletConnect, .notifications, .recents, .contacts:
            [:]
        }
    }
}
