// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemNftListScreen
import protocol Gemstone.GemNftServiceProtocol
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import Store
import SwiftUI

@MainActor
public protocol CollectionsViewable: AnyObject, Observable {
    var query: ObservableQuery<NFTRequest> { get }
    var wallet: Wallet { get }
    var screen: GemNftListScreen { get }
    var service: any GemNftServiceProtocol { get }

    var title: String { get }
    var columns: [GridItem] { get }
    var content: CollectionsContent { get }
    var emptyContentModel: EmptyContentTypeViewModel { get }

    var isPresentingReceiveSelectAssetType: SelectAssetType? { get set }

    func load() async
    func onSelectReceive()
}

public extension CollectionsViewable {
    var columns: [GridItem] {
        Array(repeating: GridItem(spacing: .medium), count: 2)
    }

    var title: String {
        screen.title.text
    }

    var offersReceive: Bool {
        screen.offersReceive
    }

    var syncsOnAppear: Bool {
        screen.syncsOnAppear
    }

    var emptyContentModel: EmptyContentTypeViewModel {
        EmptyContentTypeViewModel(type: .nfts(action: onSelectReceive))
    }

    func load() async {
        do {
            let count = try await service.sync()
            debugLog("update nfts: \(count)")
        } catch {
            debugLog("update nfts error: \(error)")
        }
    }

    func onSelectReceive() {
        isPresentingReceiveSelectAssetType = .receive(.collection)
    }
}
