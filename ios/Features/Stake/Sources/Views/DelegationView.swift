// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Style
import SwiftUI

public struct DelegationView: View {
    private let delegation: DelegationViewModel

    public init(delegation: DelegationViewModel) {
        self.delegation = delegation
    }

    public var body: some View {
        ListItemView(model: delegation.listItem)
    }
}
