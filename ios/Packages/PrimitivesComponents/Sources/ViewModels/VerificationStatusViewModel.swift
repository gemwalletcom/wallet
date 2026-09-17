// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import GemstonePrimitives
import Localization
import Primitives
import Style

public struct VerificationStatusViewModel {
    public let status: VerificationStatus

    public init(status: VerificationStatus) {
        self.status = status
    }

    public var title: String {
        status.statusTitle
    }

    public var description: String {
        status.statusDescription
    }

    public var statusStyle: TextStyle {
        status.statusStyle
    }

    public var assetImage: AssetImage {
        status.statusAssetImage
    }

    public var docsUrl: URL {
        AppUrl.docs(.tokenVerification)
    }
}
