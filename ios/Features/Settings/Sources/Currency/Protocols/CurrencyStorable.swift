// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public protocol CurrencyStorable: Sendable {
    var currency: Primitives.Currency { get set }
}
