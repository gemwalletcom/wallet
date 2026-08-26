// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemExplorerServiceProtocol
import Primitives

public struct ExplorerPreferencesMigration {
    private let service: any GemExplorerServiceProtocol
    private let defaults: UserDefaults

    public init(service: any GemExplorerServiceProtocol, defaults: UserDefaults = .standard) {
        self.service = service
        self.defaults = defaults
    }

    public func migrate() {
        for chain in Chain.allCases {
            let key = "explorer_name_\(chain.rawValue)"
            guard let name = defaults.string(forKey: key) else { continue }
            try? service.setExplorerName(chain: chain.rawValue, name: name)
            defaults.removeObject(forKey: key)
        }
    }
}
