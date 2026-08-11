// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.UrlAction
import func Gemstone.urlAction
import Primitives

enum URLParserError: Error {
    case unsupported(String)
}

public enum URLParser {
    public static func from(url: URL) throws -> URLAction {
        try from(string: url.absoluteString)
    }

    public static func from(string: String) throws -> URLAction {
        guard let action = urlAction(url: string) else {
            throw URLParserError.unsupported(string)
        }
        return try action.map()
    }
}
