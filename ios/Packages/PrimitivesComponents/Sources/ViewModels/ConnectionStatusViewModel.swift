// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemConnectionService
import Localization
import Primitives
import Style
import SwiftUI

public struct ConnectionStatusViewModel {
    private let status: ConnectionStatus

    public init(status: ConnectionStatus) {
        self.status = status
    }

    public var isVisible: Bool {
        status != .online
    }

    public static var bannerSettleDelay: Duration {
        .seconds(GemConnectionService.shared.bannerSettleDelay())
    }

    public var title: String? {
        status.bannerTitle
    }

    public var subtitle: String {
        Localized.Errors.balancesActivityOutdated
    }

    public var icon: Image {
        Images.System.exclamationmarkTriangle
    }

    public var iconColor: Color {
        Colors.orange
    }
}
