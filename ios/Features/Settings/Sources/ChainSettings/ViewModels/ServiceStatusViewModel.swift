// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Localization
import Primitives
import ServiceStatusService

@Observable
@MainActor
public final class ServiceStatusViewModel {
    private let serviceStatusService: any ServiceStatusServiceable
    private let endpoints: [ServiceEndpoint]
    private var statusStates: [ServiceStatusState]

    public init(serviceStatusService: any ServiceStatusServiceable) {
        self.serviceStatusService = serviceStatusService
        endpoints = serviceStatusService.endpoints
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
                        let milliseconds = try await service.endpointLatency(url: endpoint.url)
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
