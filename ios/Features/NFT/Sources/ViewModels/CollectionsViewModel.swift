// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemLoadState
import enum Gemstone.GemNftList
import struct Gemstone.GemNftListScreen
import protocol Gemstone.GemNftServiceProtocol
import func Gemstone.loadError
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import Store
import SwiftUI

@Observable
@MainActor
public final class CollectionsViewModel: Sendable {
    public let service: any GemNftServiceProtocol
    public let wallet: Wallet
    public let query: ObservableQuery<NFTRequest>
    private let list: GemNftList

    public var loadState: GemLoadState = .loading
    public var isPresentingToastMessage: ToastMessage?
    public var isPresentingReceiveSelectAssetType: SelectAssetType?

    public init(
        service: any GemNftServiceProtocol,
        wallet: Wallet,
        list: GemNftList,
        collectionId: String? = nil,
    ) {
        self.service = service
        self.wallet = wallet
        self.list = list
        query = ObservableQuery(NFTRequest(walletId: wallet.id, filter: collectionId.map { .collection(id: $0) } ?? .all), initialValue: [])
    }

    public var screen: GemNftListScreen {
        service.listScreen(data: query.value.map { $0.toGem() }, list: list)
    }

    public var content: CollectionsContent {
        CollectionsContent(screen)
    }

    public var columns: [GridItem] {
        Array(repeating: GridItem(spacing: .medium), count: 2)
    }

    public var emptyContentModel: EmptyContentTypeViewModel {
        EmptyContentTypeViewModel(type: EmptyContentType(.nfts, actions: [.receive: onSelectReceive]))
    }

    public func loadError(_ screen: GemNftListScreen) -> Error? {
        Gemstone.loadError(state: loadState, hasRows: screen.hasContent)
    }

    public func load() async {
        let result = await service.refresh(hasContent: screen.hasContent)
        loadState = result.state
        if let toast = result.toast {
            isPresentingToastMessage = .error(toast.text().text)
        }
    }

    public func onSelectReceive() {
        isPresentingReceiveSelectAssetType = .receive(.collection)
    }
}
