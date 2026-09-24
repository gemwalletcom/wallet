// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemPerpetualMarketSections
import func Gemstone.perpetualMarketQuery
import func Gemstone.perpetualMarketSections
import GemstonePrimitives
import Store

extension PerpetualsRequest {
    static func market(search: String) -> PerpetualsRequest {
        let query = perpetualMarketQuery(search: search)
        return PerpetualsRequest(searchQuery: query.search, limit: Int(query.limit), requiresVolume: query.requiresVolume)
    }
}

extension MappedRequest where Base == PerpetualsRequest, Value == GemPerpetualMarketSections {
    static func marketSections(search: String) -> Self {
        MappedRequest(.market(search: search)) { perpetualMarketSections(markets: $0.map { $0.toGem() }) }
    }
}
