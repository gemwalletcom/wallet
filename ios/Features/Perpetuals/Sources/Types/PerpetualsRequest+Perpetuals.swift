// Copyright (c). Gem Wallet. All rights reserved.

import func Gemstone.perpetualMarketQuery
import Store

extension PerpetualsRequest {
    static func market(search: String) -> PerpetualsRequest {
        let query = perpetualMarketQuery(search: search)
        return PerpetualsRequest(searchQuery: query.search, limit: Int(query.limit), requiresVolume: query.requiresVolume)
    }
}
