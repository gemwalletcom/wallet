// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents

@Observable
@MainActor
public final class ChainListSettingsViewModel {
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
        Localized.Settings.Networks.title
    }

    var chainsTitle: String {
        "Chains"
    }

    var connectivityModels: [ConnectivityItemViewModel] {
        connectivityEndpoints.map {
            ConnectivityItemViewModel(
                endpoint: $0,
                statusState: connectivityStatusByEndpointId[$0.id] ?? .none,
            )
        }
    }

    var emptyContent: EmptyContentTypeViewModel {
        EmptyContentTypeViewModel(type: .search(type: .networks))
    }

    var chains: [Chain] {
        AssetConfiguration.allChains.sortByRank()
    }

    func filter(_ chain: Chain, query: String) -> Bool {
        chain.asset.name.localizedCaseInsensitiveContains(query) ||
            chain.asset.symbol.localizedCaseInsensitiveContains(query) ||
            chain.rawValue.localizedCaseInsensitiveContains(query)
    }
}

// MARK: - Actions

extension ChainListSettingsViewModel {
    func fetchConnectivity() async {
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
