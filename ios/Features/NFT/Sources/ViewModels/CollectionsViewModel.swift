// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemLoadState
import struct Gemstone.GemNftListScreen
import protocol Gemstone.GemNftServiceProtocol
import GemstonePrimitives
import Localization
import Primitives
import Store
import SwiftUI

@Observable
@MainActor
public final class CollectionsViewModel: CollectionsViewable, Sendable {
    public let service: any GemNftServiceProtocol

    public let query: ObservableQuery<NFTRequest>

    public var loadState: GemLoadState = .loading

    public var isPresentingReceiveSelectAssetType: SelectAssetType?

    public let wallet: Wallet

    public init(
        service: any GemNftServiceProtocol,
        wallet: Wallet,
    ) {
        self.service = service
        self.wallet = wallet
        query = ObservableQuery(NFTRequest(walletId: wallet.id, filter: .all), initialValue: [])
    }

    public var screen: GemNftListScreen {
        service.listScreen(data: query.value.map { $0.toGem() }, list: .collections)
    }

    public var content: CollectionsContent {
        let screen = screen
        return CollectionsContent(
            items: NFTGridPosterBuilder.items(screen.items),
            unverifiedCount: screen.unverifiedRow?.countText,
        )
    }

    // MARK: - Actions
}
