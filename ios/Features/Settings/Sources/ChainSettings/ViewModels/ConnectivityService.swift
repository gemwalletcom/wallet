// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

struct ConnectivityService {
    private let session: URLSession

    init(
        session: URLSession = .shared,
    ) {
        self.session = session
    }

    func status(for endpoint: ConnectivityEndpoint) async -> ConnectivityStatusState {
        do {
            let latency = try await endpointLatency(for: endpoint.url)
            return .result(latency)
        } catch {
            return .error
        }
    }
}

// MARK: - Private

extension ConnectivityService {
    private func endpointLatency(for url: URL) async throws -> Latency {
        try await measureLatency {
            var request = URLRequest(
                url: url,
                cachePolicy: .reloadIgnoringLocalAndRemoteCacheData,
                timeoutInterval: 10,
            )
            request.httpMethod = "GET"

            let (_, response) = try await session.data(for: request)
            guard response is HTTPURLResponse else {
                throw URLError(.badServerResponse)
            }
        }
    }

    private func measureLatency(_ operation: () async throws -> Void) async throws -> Latency {
        let start = Date()
        try await operation()
        return .from(duration: Date().timeIntervalSince(start) * 1000)
    }
}
