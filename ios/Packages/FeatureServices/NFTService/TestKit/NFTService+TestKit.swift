// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemDeviceApiClient
import class Gemstone.GemNftService
import protocol Gemstone.GemNftServiceProtocol
import NativeProviderService
import NFTService
import Primitives
import Store
import StoreTestKit

public extension NFTService {
    static func mock(
        service: any GemNftServiceProtocol = GemNftService.mock(),
    ) -> NFTService {
        NFTService(service: service)
    }
}

public extension GemNftService {
    static func mock(nftStore: NFTStore = .mock()) -> GemNftService {
        GemNftService(
            api: GemDeviceApiClient(
                provider: NativeProvider(url: Constants.apiURL, requestInterceptor: EmptyRequestInterceptor()),
                baseUrl: Constants.apiURL.absoluteString,
                devicePrivateKey: Data(),
            ),
            store: GemstoneNftStore(store: nftStore),
        )
    }
}
