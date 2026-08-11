// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public enum JobStatus: Sendable {
    case complete
    case cancelled
    case retry(error: String? = nil)
}
