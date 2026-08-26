// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemApiClient
import class Gemstone.GemAssetsService
import NativeProviderService
import Primitives

public extension GemAssetsService {
    static func mock() -> GemAssetsService {
        GemAssetsService(
            api: GemApiClient(
                provider: NativeProvider(url: Constants.apiURL, requestInterceptor: EmptyRequestInterceptor()),
                baseUrl: Constants.apiURL.absoluteString,
            ),
        )
    }
}
