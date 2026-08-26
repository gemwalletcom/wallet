// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemServiceStatusProtocol
import GemstonePrimitives
import Localization
import Primitives

@Observable
@MainActor
public final class ServiceStatusViewModel {
    private let serviceStatusService: any GemServiceStatusProtocol
    private let endpoints: [ServiceEndpoint]
    private var statusStates: [ServiceStatusState]

    public init(serviceStatusService: any GemServiceStatusProtocol) {
        self.serviceStatusService = serviceStatusService
        endpoints = serviceStatusService.getEndpoints().map { $0.map() }
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
    func fetch() async {
        statusStates = Array(repeating: .loading, count: endpoints.count)

        let service = serviceStatusService
        await withTaskGroup(of: (Int, ServiceStatusState).self) { group in
            for (index, endpoint) in endpoints.enumerated() {
                group.addTask {
                    do {
                        let milliseconds = try await service.getEndpointLatency(url: endpoint.url)
                        return (index, .result(milliseconds))
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
