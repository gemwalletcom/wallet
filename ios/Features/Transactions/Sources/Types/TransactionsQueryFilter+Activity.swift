// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import func Gemstone.activityFilters
import struct Gemstone.GemActivityFilters
import enum Gemstone.GemTransactionFilter
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import Store

public extension TransactionsQueryFilter {
    static var activityDefaults: [TransactionsQueryFilter] {
        activity(chains: [], filters: [])
    }

    static var pendingActivity: [TransactionsQueryFilter] {
        let activity = activityFilters(chains: [], filters: [])
        return requestFilters(activity) + [.states(activity.pendingStates.map { $0.toPrimitives().rawValue })]
    }

    static func activity(chains: [Chain], filters: [GemTransactionFilter]) -> [TransactionsQueryFilter] {
        requestFilters(activityFilters(chains: chains.map(\.rawValue), filters: filters))
    }

    private static func requestFilters(_ activity: GemActivityFilters) -> [TransactionsQueryFilter] {
        var request: [TransactionsQueryFilter] = [.assetRankGreaterThan(activity.assetRankGreaterThan.asInt)]
        if activity.chains.isNotEmpty {
            request.append(.chains(activity.chains))
        }
        request.append(.types(activity.transactionTypes.map { $0.toPrimitives().rawValue }))
        return request
    }
}
