// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import func Gemstone.activityFilters
import enum Gemstone.GemTransactionFilter
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import Store

public extension TransactionsRequestFilter {
    static var activityDefaults: [TransactionsRequestFilter] {
        activity(chains: [], filters: [])
    }

    static func activity(chains: [Chain], filters: [GemTransactionFilter]) -> [TransactionsRequestFilter] {
        let activity = activityFilters(chains: chains.map(\.rawValue), filters: filters)
        var request: [TransactionsRequestFilter] = [.assetRankGreaterThan(activity.assetRankGreaterThan.asInt)]
        if activity.chains.isNotEmpty {
            request.append(.chains(activity.chains))
        }
        request.append(.types(activity.transactionTypes.map { $0.toPrimitives().rawValue }))
        return request
    }
}
