// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public enum PaymentSheetType: Identifiable, Equatable, Sendable {
    case dataCollection(URL)
    case quotes

    public var id: String {
        switch self {
        case let .dataCollection(url): "dataCollection-\(url)"
        case .quotes: "quotes"
        }
    }
}
