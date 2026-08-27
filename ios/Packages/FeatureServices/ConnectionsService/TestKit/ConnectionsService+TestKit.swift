// Copyright (c). Gem Wallet. All rights reserved.

@testable import ConnectionsService
import Foundation
import Preferences
import PreferencesTestKit
import Primitives
import Store
import StoreTestKit
import WalletConnectorService
import WalletConnectorServiceTestKit

public extension ConnectionsService {
    static func mock(
        connector: WalletConnectorServiceable = WalletConnectorServiceMock(),
        preferences: Preferences = .mock(),
    ) -> ConnectionsService {
        ConnectionsService(
            connector: connector,
            preferences: preferences,
        )
    }
}
