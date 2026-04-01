// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Localization
import Primitives

public extension Error {
    var networkOrNoDataDescription: String {
        isNetworkError(self) ? localizedDescription : Localized.Errors.noDataAvailable
    }
}
