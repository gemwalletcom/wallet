// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemPerpetualMarketSections
import func Gemstone.perpetualMarketQuery
import func Gemstone.perpetualMarketSections
import GemstonePrimitives
import Store

extension PerpetualsQuery {
    static func market(search: String) -> PerpetualsQuery {
        let query = perpetualMarketQuery(search: search)
        return PerpetualsQuery(searchQuery: query.search, limit: Int(query.limit), requiresVolume: query.requiresVolume)
    }
}

extension MappedQuery where Base == PerpetualsQuery, Value == GemPerpetualMarketSections {
    static func marketSections(search: String) -> Self {
        MappedQuery(.market(search: search)) { perpetualMarketSections(markets: $0.map { $0.toGem() }) }
    }
}
