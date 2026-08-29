// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemDeeplinkService
import enum Gemstone.UrlAction
import Primitives

enum URLParserError: Error {
    case invalidURL(String)
}

public enum URLParser {
    public static func from(url: URL) throws -> URLAction {
        try from(code: url.absoluteString)
    }

    public static func from(code: String) throws -> URLAction {
        guard let action = GemDeeplinkService().urlAction(url: code) else {
            throw URLParserError.invalidURL(code)
        }
        return try action.map()
    }
}
