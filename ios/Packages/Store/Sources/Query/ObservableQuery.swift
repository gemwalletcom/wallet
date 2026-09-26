// Copyright (c). Gem Wallet. All rights reserved.

import Combine
import Foundation
import GRDB
import Primitives

@Observable
@MainActor
public final class ObservableQuery<Query: DatabaseQueryable>: Sendable, BindableQuery {
    public var request: Query {
        didSet {
            guard request != oldValue else { return }
            startObservation()
        }
    }

    public internal(set) var value: Query.Value

    private var dbQueue: DatabaseQueue?
    private var cancellable: AnyCancellable?

    public init(_ request: Query, initialValue: Query.Value) {
        self.request = request
        value = initialValue
    }

    public func bind(dbQueue: DatabaseQueue) {
        guard self.dbQueue == nil else { return }
        self.dbQueue = dbQueue
        startObservation()
    }

    private func startObservation() {
        guard let dbQueue else { return }
        let request = request

        cancellable = ValueObservation
            .tracking { db in try request.fetch(db) }
            .publisher(in: dbQueue, scheduling: .immediate)
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
