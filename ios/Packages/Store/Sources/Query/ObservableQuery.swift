// Copyright (c). Gem Wallet. All rights reserved.

import Combine
import Foundation
import GRDB
import Primitives

@Observable
@MainActor
public final class ObservableQuery<Query: DatabaseQueryable>: Sendable, BindableQuery {
    public var request: Query {
        get { observedRequest }
        set { observe(newValue, scheduling: .immediate) }
    }

    public internal(set) var value: Query.Value

    private var observedRequest: Query
    private var dbPool: DatabasePool?
    private var cancellable: AnyCancellable?

    public init(_ request: Query, initialValue: Query.Value) {
        observedRequest = request
        value = initialValue
    }

    public func bind(dbPool: DatabasePool) {
        guard self.dbPool == nil else { return }
        self.dbPool = dbPool
        startObservation(scheduling: .immediate)
    }

    private func observe(_ request: Query, scheduling: some ValueObservationScheduler) {
        guard request != observedRequest else { return }
        observedRequest = request
        startObservation(scheduling: scheduling)
    }

    private func startObservation(scheduling: some ValueObservationScheduler) {
        guard let dbPool else { return }
        let request = observedRequest

        cancellable = ValueObservation
            .tracking { db in try request.fetch(db) }
            .publisher(in: dbPool, scheduling: scheduling)
            .sink(
                receiveCompletion: { completion in
                    if case let .failure(error) = completion {
                        debugLog("ObservableQuery<\(Query.self)> error: \(error)")
                    }
                },
                receiveValue: { [weak self] newValue in
                    self?.value = newValue
                },
            )
    }
}

public extension ObservableQuery {
    func setRequestInBackground(_ request: Query) {
        observe(request, scheduling: .async(onQueue: .main))
    }
}

public extension ObservableQuery where Query: LimitedQuery {
    func setLimit(_ limit: Int) {
        setRequestInBackground(observedRequest.with(limit: limit))
    }
}
