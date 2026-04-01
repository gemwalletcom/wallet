// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public extension Bundle {
    var releaseVersionNumber: String {
        infoDictionary?["CFBundleShortVersionString"] as? String ?? ""
    }

    var buildVersionNumber: Int {
        Int((infoDictionary?["CFBundleVersion"] as? String ?? "")) ?? 0
    }
}
