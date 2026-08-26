// Copyright (c). Gem Wallet. All rights reserved.

import ChainService
import ChainServiceTestKit
import Foundation
import class Gemstone.GemStaticApiClient
import class Gemstone.GemStaticAssetsService
import NativeProviderService
import Primitives
import StakeService
import Store
import StoreTestKit

public extension StakeService {
    static func mock(
        store: StakeStore = .mock(),
        addressStore: AddressStore = .mock(),
        chainServiceFactory: any ChainServiceFactorable = ChainServiceFactoryMock(),
        assetsService: GemStaticAssetsService = .mock(),
    ) -> Self {
        StakeService(
            store: store,
            addressStore: addressStore,
            chainServiceFactory: chainServiceFactory,
            assetsService: assetsService,
        )
    }
}

public extension GemStaticAssetsService {
    static func mock() -> GemStaticAssetsService {
        GemStaticAssetsService(
            api: GemStaticApiClient(
                provider: NativeProvider(url: Constants.assetsURL, requestInterceptor: EmptyRequestInterceptor()),
                baseUrl: Constants.assetsURL.absoluteString,
            ),
        )
    }
}
