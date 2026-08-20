// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import InfoSheet

public enum PaymentSheetType: Identifiable, Equatable, Sendable {
    case info(InfoSheetType)
    case dataCollection(URL)
    case quotes

    public var id: String {
        switch self {
        case let .info(type): "info-\(type.id)"
        case let .dataCollection(url): "dataCollection-\(url)"
        case .quotes: "quotes"
        }
    }
}
