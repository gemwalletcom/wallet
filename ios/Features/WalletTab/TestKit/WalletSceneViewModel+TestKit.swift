// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemNftService
import GemstoneServicesTestKit
import Foundation
import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import GemstoneServices
import GemstonePrimitivesTestKit
import NFT
import WalletTab

public extension WalletSceneViewModel {
    static func mock(wallet: Wallet = .mock()) -> WalletSceneViewModel {
        WalletSceneViewModel(
            service: GemWalletHomeServiceMock(),
            observablePreferences: .mock(),
            collectionsModel: CollectionsViewModel(service: GemNftService.mock(), wallet: wallet),
            wallet: wallet,
            isPresentingSelectedAssetInput: .constant(.none),
            isPresentingWallets: .constant(false),
        )
    }
}
