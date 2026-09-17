// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import Primitives
import PrimitivesComponents

public enum TransactionsSheetType: Identifiable {
    case filter
    case selectAsset(SelectAssetType)
    case addContact(AddContactType)
    case addressDetails(ChainAddress)

    public var id: String {
        switch self {
        case .filter: "filter"
        case let .selectAsset(type): "selectAsset-\(type.id)"
        case let .addContact(type): "addContact-\(type.id)"
        case let .addressDetails(chainAddress): "addressDetails-\(chainAddress.chain.rawValue)-\(chainAddress.address)"
        }
    }
}
