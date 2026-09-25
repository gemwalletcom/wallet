// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemNftList
import class Gemstone.GemNftService
import GemstoneServicesTestKit
import NFT
import Primitives
import PrimitivesTestKit

public extension CollectionsViewModel {
    @MainActor
    static func mock(wallet: Wallet = .mock(), list: GemNftList = .collections, collectionId: String? = nil) -> CollectionsViewModel {
        CollectionsViewModel(service: GemNftService.mock(), wallet: wallet, list: list, collectionId: collectionId)
    }
}
