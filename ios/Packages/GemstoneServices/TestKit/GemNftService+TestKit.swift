// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemDeviceApiClient
import class Gemstone.GemDeviceKeyService
import class Gemstone.GemAvatarService
import class Gemstone.GemCollectibleService
import class Gemstone.GemExplorerService
import class Gemstone.GemNftService
import protocol Gemstone.GemNftServiceProtocol
import NativeProviderService
import GemstonePrimitivesTestKit
import GemstoneServices
import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit

public extension GemCollectibleService {
    static func mock(nftStore: NFTStore = .mock(), explorer: GemExplorerService = .mock()) -> GemCollectibleService {
        GemCollectibleService(
            nfts: GemNftService.mock(nftStore: nftStore),
            avatars: GemAvatarService(wallets: GemstoneWalletStore(store: .mock()), files: GemstoneFileStore(), provider: NativeProvider()),
            explorer: explorer,
        )
    }
}

public extension GemNftService {
    static func mock(nftStore: NFTStore = .mock()) -> GemNftService {
        GemNftService(
            api: GemDeviceApiClient(
                provider: NativeProvider(),
                deviceKey: GemDeviceKeyService(store: GemSecureStoreMock()),
            ),
            store: GemstoneNftStore(store: nftStore),
            session: .mock(),
        )
    }
}
