// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Localization

@Observable
@MainActor
public final class ServiceStatusViewModel {
    private let connectivityService: ConnectivityService
    private let connectivityEndpoints: [ConnectivityEndpoint]
    private var connectivityStatusByEndpointId: [String: ConnectivityStatusState] = [:]

    public convenience init() {
        self.init(
            connectivityService: ConnectivityService(),
            connectivityEndpoints: ConnectivityEndpoint.defaultEndpoints,
        )
    }

    init(
        connectivityService: ConnectivityService,
        connectivityEndpoints: [ConnectivityEndpoint],
    ) {
        self.connectivityService = connectivityService
        self.connectivityEndpoints = connectivityEndpoints
    }

    var title: String {
        Localized.Transaction.status
    }

    var connectivityModels: [ConnectivityItemViewModel] {
        connectivityEndpoints.map {
            ConnectivityItemViewModel(
                endpoint: $0,
                statusState: connectivityStatusByEndpointId[$0.id] ?? .none,
            )
        }
    }
}

// MARK: - Actions

extension ServiceStatusViewModel {
    func fetch() async {
        connectivityStatusByEndpointId = [:]

        let service = connectivityService
        await withTaskGroup(of: (String, ConnectivityStatusState).self) { group in
            for endpoint in connectivityEndpoints {
                group.addTask {
                    await (endpoint.id, service.status(for: endpoint))
                }
            }

            for await (endpointId, status) in group {
                connectivityStatusByEndpointId[endpointId] = status
            }
        }
    }
}
