// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemServiceEndpoint
import protocol Gemstone.GemServiceStatusProtocol
import GemstonePrimitives
import Localization
import Primitives

@Observable
@MainActor
public final class ServiceStatusViewModel {
    private let serviceStatusService: any GemServiceStatusProtocol
    private let endpoints: [GemServiceEndpoint]
    private var statusStates: [ServiceStatusState]

    public init(serviceStatusService: any GemServiceStatusProtocol) {
        self.serviceStatusService = serviceStatusService
        endpoints = serviceStatusService.getEndpoints()
        statusStates = Array(repeating: .loading, count: endpoints.count)
    }

    var title: String {
        Localized.Transaction.status
    }

    var itemModels: [ServiceStatusItemViewModel] {
        zip(endpoints, statusStates).map {
            ServiceStatusItemViewModel(endpoint: $0, statusState: $1)
        }
    }
}

// MARK: - Actions

extension ServiceStatusViewModel {
    func load() async {
        statusStates = Array(repeating: .loading, count: endpoints.count)

        let service = serviceStatusService
        await withTaskGroup(of: (Int, ServiceStatusState).self) { group in
            for (index, endpoint) in endpoints.enumerated() {
                group.addTask {
                    do {
                        return try await (index, .result(service.getEndpointLatency(url: endpoint.url).map()))
                    } catch {
                        return (index, .error)
                    }
                }
            }

            for await (index, state) in group {
                statusStates[index] = state
            }
        }
    }
}
