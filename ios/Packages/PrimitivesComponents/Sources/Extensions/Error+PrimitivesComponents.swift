// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemServiceError
import Localization
import Primitives

public extension Error {
    var networkOrNoDataDescription: String {
        if let error = self as? GemServiceError, error == .Offline {
            return error.text().text
        }
        return isNetworkError(self) ? localizedDescription : Localized.Errors.noDataAvailable
    }
}
