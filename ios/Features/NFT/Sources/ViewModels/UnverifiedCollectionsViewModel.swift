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
public final class UnverifiedCollectionsViewModel: CollectionsViewable, Sendable {
    public let service: any GemNftServiceProtocol

    public let wallet: Wallet
    public let query: ObservableQuery<NFTRequest>

    public var loadState: GemLoadState = .loading

    public var isPresentingReceiveSelectAssetType: SelectAssetType?

    public init(service: any GemNftServiceProtocol, wallet: Wallet) {
        self.service = service
        self.wallet = wallet
        query = ObservableQuery(NFTRequest(walletId: wallet.id, filter: .all), initialValue: [])
    }

    public var screen: GemNftListScreen {
        service.listScreen(data: query.value.map { $0.toGem() }, list: .unverified)
    }

    public var content: CollectionsContent {
        CollectionsContent(items: NFTGridPosterBuilder.items(screen.items))
    }
}
