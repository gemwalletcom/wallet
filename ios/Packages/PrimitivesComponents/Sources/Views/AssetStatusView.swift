// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Primitives
import SwiftUI

public struct AssetStatusView: View {
    private let status: VerificationStatus
    private let action: () -> Void

    public init(
        status: VerificationStatus,
        action: @escaping () -> Void,
    ) {
        self.status = status
        self.action = action
    }

    public var body: some View {
        NavigationCustomLink(with:
            ListItemImageView(
                title: Localized.Transaction.status,
                subtitle: status.statusTitle,
                subtitleStyle: status.statusStyle,
                assetImage: status.statusAssetImage,
                infoAction: action,
            )) {
                action()
            }
    }
}
