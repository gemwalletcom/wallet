// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

struct AppResolver: Sendable {
    let storages: Storages
    let navigation: NavigationStateManager
    let services: Services

    static let main: AppResolver = .init()

    private init(
        storages: Storages = Storages(),
        factory: ServicesFactory = ServicesFactory(),
        navigation: NavigationStateManager = NavigationStateManager(),
    ) {
        self.storages = storages
        self.navigation = navigation
        services = factory.makeServices(storages: storages, navigation: navigation)
    }
}
