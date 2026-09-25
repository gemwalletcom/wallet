// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.FiatQuoteUrl
import protocol Gemstone.GemFiatQuoteServiceProtocol
import struct Gemstone.GemFiatSession
import Primitives

public extension GemFiatQuoteServiceProtocol {
    func newSession(type: FiatQuoteType, amount: Int?) -> GemFiatSession {
        newSession(quoteType: type.toGem(), amount: amount.map { UInt32($0) })
    }

    func quoteUrl(asset: Asset, quoteId: String) async throws -> FiatQuoteUrl {
        try await quoteUrl(assetId: asset.id.identifier, quoteId: quoteId)
    }
}

public extension GemFiatSession {
    var type: FiatQuoteType {
        quoteType.toPrimitives()
    }
}
