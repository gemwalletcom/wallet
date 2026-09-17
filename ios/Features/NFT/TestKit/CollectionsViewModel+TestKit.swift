// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemNftService
import GemstoneServicesTestKit
import NFT
import Primitives
import PrimitivesTestKit

public extension CollectionsViewModel {
    @MainActor
    static func mock(wallet: Wallet = .mock()) -> CollectionsViewModel {
        CollectionsViewModel(service: GemNftService.mock(), wallet: wallet)
    }
}
