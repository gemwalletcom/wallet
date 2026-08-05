// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

/// Thrown when a sheet closes without producing a value.
public enum SheetDismissal: Error, Equatable {
    case cancelled
}
