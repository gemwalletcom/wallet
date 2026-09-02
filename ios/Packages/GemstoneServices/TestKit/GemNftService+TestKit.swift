// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemDeviceApiClient
import class Gemstone.GemDeviceKeyService
import class Gemstone.GemNftService
import protocol Gemstone.GemNftServiceProtocol
import NativeProviderService
import GemstonePrimitivesTestKit
import GemstoneServices
import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit

public extension GemNftService {
    static func mock(nftStore: NFTStore = .mock()) -> GemNftService {
        GemNftService(
            api: GemDeviceApiClient(
                provider: NativeProvider(url: Constants.apiURL),
                baseUrl: Constants.apiURL.absoluteString,
                deviceKey: GemDeviceKeyService(store: GemSecureStoreMock()),
            ),
            store: GemstoneNftStore(store: nftStore),
        )
    }
}
