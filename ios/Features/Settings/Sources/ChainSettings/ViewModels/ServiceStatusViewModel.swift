// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemLatencyStatus
import struct Gemstone.GemServiceEndpoint
import protocol Gemstone.GemServiceStatusProtocol
import Localization

@Observable
@MainActor
public final class ServiceStatusViewModel {
    private let service: any GemServiceStatusProtocol
    private let endpoints: [GemServiceEndpoint]
    private var statusStates: [GemLatencyStatus]

    public init(service: any GemServiceStatusProtocol) {
        self.service = service
        endpoints = service.getEndpoints()
        statusStates = Array(repeating: .loading, count: endpoints.count)
    }

    var title: String {
        Localized.Transaction.status
    }

    var itemModels: [ServiceStatusItemViewModel] {
        zip(endpoints, statusStates).map {
            ServiceStatusItemViewModel(endpoint: $0, status: $1)
        }
    }
}

// MARK: - Actions

extension ServiceStatusViewModel {
    func load() async {
        statusStates = Array(repeating: .loading, count: endpoints.count)

        let service = service
        await withTaskGroup(of: (Int, GemLatencyStatus).self) { group in
            for (index, endpoint) in endpoints.enumerated() {
                group.addTask {
                    await (index, service.getEndpointStatus(url: endpoint.url))
                }
            }

            for await (index, state) in group {
                statusStates[index] = state
            }
        }
    }
}
