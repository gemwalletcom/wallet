// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemDeeplinkService
import enum Gemstone.UrlAction
import Primitives

enum URLParserError: Error {
    case invalidURL(String)
}

public struct URLParser: Sendable {
    private let deeplinkService: GemDeeplinkService

    public init(deeplinkService: GemDeeplinkService) {
        self.deeplinkService = deeplinkService
    }

    public func from(url: URL) throws -> URLAction {
        try from(code: url.absoluteString)
    }

    public func from(code: String) throws -> URLAction {
        guard let action = deeplinkService.urlAction(url: code) else {
            throw URLParserError.invalidURL(code)
        }
        return try action.map()
    }
}
