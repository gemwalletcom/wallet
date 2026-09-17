// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation

public struct DelegationRowViewModel: Identifiable {
    public enum Action {
        case plain
        case url(URL)
        case claimRewards
    }

    public let id: String
    public let action: Action
    public let model: ListItemModel
}
