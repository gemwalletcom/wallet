// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public enum SignMessageSheet: Identifiable, Sendable {
    case payloadDetails
    case addressDetails(ChainAddress)

    public var id: String {
        switch self {
        case .payloadDetails: "payload-details"
        case let .addressDetails(chainAddress): "address-details-\(chainAddress.chain.rawValue)-\(chainAddress.address)"
        }
    }
}
