// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesComponents

public enum TransactionsSheetType: Identifiable {
    case filter
    case selectAsset(SelectAssetType)
    case addContact(AddContactType)

    public var id: String {
        switch self {
        case .filter: "filter"
        case let .selectAsset(type): "selectAsset-\(type.id)"
        case let .addContact(type): "addContact-\(type.id)"
        }
    }
}
