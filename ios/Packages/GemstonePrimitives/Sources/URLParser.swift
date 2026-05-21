// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.UrlAction
import func Gemstone.urlAction
import Primitives

enum URLParserError: Error {
    case invalidURL(URL)
}

public enum URLParser {
    public static func from(url: URL) throws -> URLAction {
        guard let action = urlAction(url: url.absoluteString) else {
            throw URLParserError.invalidURL(url)
        }
        return try action.map()
    }
}
